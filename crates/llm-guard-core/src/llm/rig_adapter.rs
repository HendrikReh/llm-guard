use super::{LlmClient, LlmSettings, ProviderKind};
use crate::scanner::{LlmVerdict, ScanReport};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use rig::client::CompletionClient;
use rig::completion::message::AssistantContent;
use rig::completion::{CompletionError, CompletionModelDyn};
use rig::providers::azure::AzureOpenAIAuth;
use rig::providers::{anthropic, azure, openai};
use rig::OneOrMany;
use serde::Deserialize;
use serde_json::json;
use std::env;

const DEFAULT_OPENAI_MODEL: &str = "gpt-4o-mini";
const DEFAULT_ANTHROPIC_MODEL: &str = "claude-3-5-sonnet-latest";
const MAX_OUTPUT_TOKENS: u64 = 200;
const TEMPERATURE: f64 = 0.1;
const SYSTEM_PROMPT: &str = "You are an application security assistant. Analyze prompt-injection scan results and respond with strict JSON: {\"label\": \"safe|suspicious|malicious\", \"rationale\": \"...\", \"mitigation\": \"...\"}. The mitigation should advise remediation steps.";

struct RigCompletionConfig {
    provider_label: &'static str,
    temperature: Option<f64>,
    max_tokens: u64,
    force_json_mime: bool,
}

pub struct RigLlmClient {
    model: Box<dyn CompletionModelDyn + Send + Sync>,
    config: RigCompletionConfig,
    model_id: String,
}

impl RigLlmClient {
    pub fn for_kind(kind: ProviderKind, settings: &LlmSettings) -> Result<Box<dyn LlmClient>> {
        match kind {
            ProviderKind::OpenAi => Ok(Box::new(Self::new_openai(settings)?)),
            ProviderKind::Anthropic => Ok(Box::new(Self::new_anthropic(settings)?)),
            ProviderKind::Gemini => {
                bail!("Gemini provider should use standalone client, not rig adapter")
            }
            ProviderKind::Azure => Ok(Box::new(Self::new_azure(settings)?)),
            ProviderKind::Noop | ProviderKind::Rig => {
                bail!("rig adapter does not support provider `{kind:?}` yet")
            }
        }
    }

    fn new_openai(settings: &LlmSettings) -> Result<Self> {
        if settings.api_key.trim().is_empty() {
            bail!("OpenAI API key must be provided via LLM_GUARD_API_KEY");
        }

        let mut builder = openai::Client::builder(&settings.api_key);
        if let Some(endpoint) = settings.endpoint.as_deref() {
            builder = builder.base_url(endpoint);
        }
        let client = builder.build();

        let model_id = settings
            .model
            .clone()
            .unwrap_or_else(|| DEFAULT_OPENAI_MODEL.to_string());

        let model: Box<dyn CompletionModelDyn + Send + Sync> =
            Box::new(client.completion_model(&model_id));

        Ok(Self::from_model(model, "openai", model_id, None, false))
    }

    fn new_anthropic(settings: &LlmSettings) -> Result<Self> {
        if settings.api_key.trim().is_empty() {
            bail!("Anthropic API key must be provided via LLM_GUARD_API_KEY");
        }

        let mut builder = anthropic::ClientBuilder::new(&settings.api_key);
        if let Some(endpoint) = settings.endpoint.as_deref() {
            builder = builder.base_url(endpoint);
        }
        if let Some(version) = settings.api_version.as_deref() {
            builder = builder.anthropic_version(version);
        }
        let client = builder
            .build()
            .context("failed to build anthropic rig client")?;

        let model_id = settings
            .model
            .clone()
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_ANTHROPIC_MODEL.to_string());

        let model: Box<dyn CompletionModelDyn + Send + Sync> =
            Box::new(client.completion_model(&model_id));

        Ok(Self::from_model(
            model,
            "anthropic",
            model_id,
            Some(TEMPERATURE),
            false,
        ))
    }

    // Note: Gemini support removed from rig adapter due to deserialization issues.
    // Gemini now uses a standalone HTTP client implementation (see gemini.rs).

    fn new_azure(settings: &LlmSettings) -> Result<Self> {
        if settings.api_key.trim().is_empty() {
            bail!("Azure OpenAI API key must be provided via LLM_GUARD_API_KEY");
        }

        let Some(endpoint) = settings.endpoint.as_deref() else {
            bail!("Azure provider requires --endpoint or LLM_GUARD_ENDPOINT to be set");
        };

        let deployment = settings
            .deployment
            .as_ref()
            .or(settings.model.as_ref())
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string());

        let Some(deployment) = deployment else {
            bail!(
                "Azure provider requires --deployment/LLM_GUARD_DEPLOYMENT or --model/LLM_GUARD_MODEL to specify the deployment name"
            );
        };

        let auth = AzureOpenAIAuth::ApiKey(settings.api_key.clone());
        let mut builder = azure::Client::builder(auth, endpoint);
        if let Some(version) = settings.api_version.as_deref() {
            builder = builder.api_version(version);
        }
        let client = builder.build();

        let model: Box<dyn CompletionModelDyn + Send + Sync> =
            Box::new(client.completion_model(&deployment));

        Ok(Self::from_model(
            model,
            "azure",
            deployment,
            Some(TEMPERATURE),
            false,
        ))
    }

    fn from_model(
        model: Box<dyn CompletionModelDyn + Send + Sync>,
        provider_label: &'static str,
        model_id: String,
        temperature: Option<f64>,
        force_json_mime: bool,
    ) -> Self {
        Self {
            model,
            config: RigCompletionConfig {
                provider_label,
                temperature,
                max_tokens: MAX_OUTPUT_TOKENS,
                force_json_mime,
            },
            model_id,
        }
    }
}

#[async_trait]
impl LlmClient for RigLlmClient {
    async fn enrich(&self, input: &str, report: &ScanReport) -> Result<LlmVerdict> {
        let prompt = format!(
            "You are validating a prompt injection scan. Respond strictly with a JSON object using keys 'label', 'rationale', and 'mitigation'.\nInput excerpt:\n{}\n\nScore: {:.1} ({:?})\nTop findings: {}\n",
            truncate(input, 2000),
            report.risk_score,
            report.risk_band,
            serde_json::to_string(&report.findings).unwrap_or_default()
        );

        let mut builder = self
            .model
            .completion_request(prompt.into())
            .preamble(SYSTEM_PROMPT.to_string())
            .max_tokens(self.config.max_tokens);

        if let Some(temp) = self.config.temperature {
            builder = builder.temperature(temp);
        }

        if self.config.force_json_mime {
            builder = builder.additional_params(json!({
                "generationConfig": {
                    "responseMimeType": "application/json"
                }
            }));
        }

        if self.config.provider_label == "openai" {
            // Use simple json_object format instead of json_schema for better compatibility
            // with reasoning models like gpt-5
            builder = builder.additional_params(json!({
                "response_format": {
                    "type": "json_object"
                }
            }));
        }
        // Note: Gemini function calling removed due to rig compatibility issues
        // Gemini will rely on prompt instructions for JSON formatting

        let request = builder.build();

        let response = match self.model.completion(request).await {
            Ok(response) => response,
            Err(CompletionError::ResponseError(text))
                if text.contains("no message") && text.contains("empty") =>
            {
                tracing::warn!(
                    "rig {} completion returned empty response; falling back",
                    self.config.provider_label
                );
                return Ok(fallback_verdict(self.config.provider_label));
            }
            Err(err) => {
                let failure: Result<_, CompletionError> = Err(err);
                return failure.with_context(|| {
                    format!(
                        "rig {} completion request failed for model {}",
                        self.config.provider_label, self.model_id
                    )
                });
            }
        };

        let choice = response.choice;
        let verdict = verdict_from_choice(choice, self.config.provider_label, &self.model_id)?;

        Ok(LlmVerdict {
            label: verdict.label,
            rationale: verdict.rationale,
            mitigation: verdict.mitigation,
        })
    }
}

#[derive(Debug, Deserialize)]
struct ModelVerdict {
    label: String,
    rationale: String,
    mitigation: String,
}

fn fallback_verdict(provider: &str) -> LlmVerdict {
    LlmVerdict {
        label: "unknown".into(),
        rationale: format!(
            "Provider `{}` returned no textual content; health check recorded response metadata only.",
            provider
        ),
        mitigation: "Inspect provider logs or retry with a model that emits textual output.".into(),
    }
}

fn parse_verdict_json(payload: &str, provider_label: &str, model_id: &str) -> Result<ModelVerdict> {
    match serde_json::from_str::<ModelVerdict>(payload) {
        Ok(verdict) => Ok(verdict),
        Err(_first_err) => {
            let sanitized = sanitize_json_strings(payload);
            if sanitized != payload {
                if let Ok(verdict) = serde_json::from_str::<ModelVerdict>(&sanitized) {
                    return Ok(verdict);
                }
            }

            let value: serde_json::Value = match json5::from_str(&sanitized) {
                Ok(value) => value,
                Err(_) => {
                    debug_log_payload(provider_label, payload);
                    tracing::warn!(
                        "rig {} response from model {} could not be parsed even with relaxed JSON; using fallback",
                        provider_label, model_id
                    );
                    return Ok(fallback_model_verdict(provider_label));
                }
            };
            match serde_json::from_value(value) {
                Ok(verdict) => Ok(verdict),
                Err(_) => {
                    debug_log_payload(provider_label, payload);
                    tracing::warn!(
                        "rig {} response from model {} could not be coerced into verdict schema; using fallback",
                        provider_label, model_id
                    );
                    Ok(fallback_model_verdict(provider_label))
                }
            }
        }
    }
}

fn fallback_model_verdict(provider: &str) -> ModelVerdict {
    ModelVerdict {
        label: "unknown".into(),
        rationale: format!(
            "Provider `{}` returned a verdict that could not be parsed into the expected JSON structure.",
            provider
        ),
        mitigation: "Review provider output or adjust prompt/response parsing schema.".into(),
    }
}

fn sanitize_json_strings(payload: &str) -> String {
    let mut result = String::with_capacity(payload.len());
    let mut in_string = false;
    let mut escape = false;

    for ch in payload.chars() {
        if in_string {
            if escape {
                result.push(ch);
                escape = false;
            } else {
                match ch {
                    '\\' => {
                        result.push(ch);
                        escape = true;
                    }
                    '"' => {
                        result.push(ch);
                        in_string = false;
                    }
                    '\n' => {
                        result.push('\\');
                        result.push('n');
                    }
                    _ => result.push(ch),
                }
            }
        } else {
            result.push(ch);
            if ch == '"' {
                in_string = true;
            }
        }
    }

    if in_string {
        result.push('"');
    }

    let open_braces = result.chars().filter(|&c| c == '{').count();
    let close_braces = result.chars().filter(|&c| c == '}').count();
    for _ in close_braces..open_braces {
        result.push('}');
    }

    let open_brackets = result.chars().filter(|&c| c == '[').count();
    let close_brackets = result.chars().filter(|&c| c == ']').count();
    for _ in close_brackets..open_brackets {
        result.push(']');
    }

    result
}

fn debug_log_payload(provider: &str, payload: &str) {
    if debug_enabled() {
        tracing::warn!("rig {} raw verdict payload: {}", provider, payload);
    }
}

fn debug_enabled() -> bool {
    matches!(env::var("LLM_GUARD_DEBUG"), Ok(val) if !val.is_empty() && val != "0")
}

fn debug_log_choice(provider: &str, choice: &rig::OneOrMany<AssistantContent>) {
    if !debug_enabled() {
        return;
    }

    match serde_json::to_string_pretty(choice) {
        Ok(json) => tracing::warn!("rig {} assistant choice payload: {}", provider, json),
        Err(err) => tracing::warn!(
            "rig {} assistant choice payload could not be serialised: {}",
            provider,
            err
        ),
    }
}

fn verdict_from_choice(
    choice: OneOrMany<AssistantContent>,
    provider_label: &str,
    model_id: &str,
) -> Result<ModelVerdict> {
    debug_log_choice(provider_label, &choice);

    let content = choice
        .clone()
        .into_iter()
        .filter_map(|segment| match segment {
            AssistantContent::Text(text) => Some(text.text),
            AssistantContent::Reasoning(reasoning) => Some(reasoning.reasoning.join("\n")),
            AssistantContent::ToolCall(tool) => {
                serde_json::to_string(&tool.function.arguments).ok()
            }
        })
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    let trimmed = content.trim();

    if debug_enabled() && !trimmed.is_empty() {
        tracing::warn!("rig {} extracted content: {}", provider_label, trimmed);
    }

    if trimmed.is_empty() {
        tracing::warn!(
            "rig {} response did not include textual content; returning fallback verdict",
            provider_label
        );
        return Ok(fallback_model_verdict(provider_label));
    }

    let json_payload = extract_json_payload(trimmed);
    parse_verdict_json(&json_payload, provider_label, model_id)
}

fn truncate(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    input.chars().take(max_chars).collect::<String>() + "…"
}

fn extract_json_payload(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(stripped) = strip_code_fence(trimmed) {
        return stripped;
    }
    trimmed.to_string()
}

fn strip_code_fence(input: &str) -> Option<String> {
    let mut trimmed = input.trim();
    if !trimmed.starts_with("```") {
        return None;
    }
    trimmed = trimmed.trim_start_matches("```");
    trimmed = trimmed.trim_start_matches(|c: char| c.is_ascii_whitespace());
    if let Some(rest) = trimmed.strip_prefix("json") {
        trimmed = rest.trim_start_matches(|c: char| c.is_ascii_whitespace());
    }
    trimmed = trimmed.trim_start_matches('\n');
    let end = trimmed.rfind("```").unwrap_or(trimmed.len());
    let fenced = &trimmed[..end];
    Some(fenced.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rig::{
        completion::message::{AssistantContent, Text},
        OneOrMany,
    };

    fn openai_settings() -> LlmSettings {
        LlmSettings {
            provider: "openai".into(),
            api_key: "test-key".into(),
            endpoint: Some("https://example.com".into()),
            model: Some("gpt-test".into()),
            deployment: None,
            project: None,
            workspace: None,
            timeout_secs: Some(30),
            max_retries: 0,
            api_version: None,
        }
    }

    #[test]
    fn openai_builder_requires_api_key() {
        let mut settings = openai_settings();
        settings.api_key.clear();
        let result = RigLlmClient::new_openai(&settings);
        assert!(result.is_err());
        let message = result.err().unwrap().to_string();
        assert!(message.contains("API key"));
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        ignore = "reqwest default TLS stack unavailable in sandbox"
    )]
    fn openai_builder_sets_model_id() {
        let settings = openai_settings();
        let client = RigLlmClient::new_openai(&settings).expect("client should be constructed");
        assert_eq!(client.config.provider_label, "openai");
        assert_eq!(client.model_id, "gpt-test");
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        ignore = "reqwest default TLS stack unavailable in sandbox"
    )]
    fn openai_builder_defaults_model_when_missing() {
        let mut settings = openai_settings();
        settings.model = None;
        let client = RigLlmClient::new_openai(&settings).expect("client should be constructed");
        assert_eq!(client.model_id, DEFAULT_OPENAI_MODEL);
    }

    fn azure_settings() -> LlmSettings {
        LlmSettings {
            provider: "azure".into(),
            api_key: "azure-key".into(),
            endpoint: Some("https://example.openai.azure.com".into()),
            model: Some("deployment-name".into()),
            deployment: None,
            project: None,
            workspace: None,
            timeout_secs: Some(30),
            max_retries: 0,
            api_version: Some("2024-02-15-preview".into()),
        }
    }

    #[test]
    fn azure_requires_endpoint() {
        let mut settings = azure_settings();
        settings.endpoint = None;
        let err = RigLlmClient::new_azure(&settings)
            .err()
            .expect("missing endpoint should error");
        assert!(err.to_string().to_lowercase().contains("endpoint"));
    }

    #[test]
    fn azure_requires_deployment() {
        let mut settings = azure_settings();
        settings.model = None;
        settings.deployment = None;
        let err = RigLlmClient::new_azure(&settings)
            .err()
            .expect("missing deployment should error");
        assert!(err.to_string().to_lowercase().contains("deployment"));
    }

    #[test]
    fn extract_json_handles_code_fence() {
        let raw = "```json\n{\"label\":\"safe\"}\n```";
        let sanitized = extract_json_payload(raw);
        assert_eq!(sanitized, "{\"label\":\"safe\"}");
    }

    #[test]
    fn fallback_verdict_produces_unknown_label() {
        let verdict = fallback_verdict("openai");
        assert_eq!(verdict.label, "unknown");
        assert!(verdict.rationale.contains("openai"));
    }

    #[test]
    fn parse_verdict_allows_multiline_strings() {
        let payload = "{\n  \"label\": \"safe\",\n  \"rationale\": \"Line one\nLine two\",\n  \"mitigation\": \"None\"\n}";
        let verdict = parse_verdict_json(payload, "anthropic", "test-model")
            .expect("should parse multiline JSON payload");
        assert_eq!(verdict.label, "safe");
        assert!(verdict.rationale.contains("Line two"));
    }

    #[test]
    fn sanitize_closes_unterminated_strings() {
        let payload = "{\n  \"mitigation\": \"Line one\nLine two";
        let sanitized = sanitize_json_strings(payload);
        assert!(sanitized.ends_with("\"}"));
        assert!(sanitized.contains("\\n"));
    }

    fn json_body_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("{\"label\":\"safe\"}".to_string()),
            Just("{\"label\":\"suspicious\",\"mitigation\":\"Review\"}".to_string()),
            Just("{\n  \"label\": \"malicious\",\n  \"rationale\": \"Nested\"\n}".to_string())
        ]
    }

    proptest! {
        #[test]
        fn extract_json_payload_strips_known_fences(body in json_body_strategy()) {
            let fenced = format!("```json\n{}\n```", body);
            let payload = extract_json_payload(&fenced);
            prop_assert_eq!(payload, body.trim());

            let fenced_caps = format!(" ``` {} ``` ", body);
            let payload_caps = extract_json_payload(&fenced_caps);
            prop_assert_eq!(payload_caps, body.trim());
        }

        #[test]
        fn extract_json_payload_preserves_unfenced(text in json_body_strategy()) {
            let decorated = format!("\n  {}\n ", text);
            let payload = extract_json_payload(&decorated);
            prop_assert_eq!(payload, text.trim());
        }
    }

    #[test]
    fn fallback_model_verdict_contains_provider() {
        let verdict = fallback_model_verdict("anthropic");
        assert_eq!(verdict.label, "unknown");
        assert!(verdict.rationale.contains("anthropic"));
        assert!(!verdict.mitigation.is_empty());
    }

    #[test]
    fn parse_verdict_returns_fallback_for_invalid_json() {
        let verdict =
            parse_verdict_json("not-json", "openai", "gpt-test").expect("should fallback");
        assert_eq!(verdict.label, "unknown");
        assert!(verdict.rationale.contains("openai"));
    }

    #[test]
    fn truncate_adds_ellipsis_when_exceeding_limit() {
        let long = "abcdefghijklmnopqrstuvwxyz";
        let truncated = truncate(long, 10);
        assert!(truncated.ends_with('…'));
        assert_eq!(truncated.chars().count(), 11);
    }

    #[test]
    fn verdict_from_choice_parses_valid_json() {
        let choice = OneOrMany::one(AssistantContent::Text(Text {
            text: "{\"label\":\"malicious\",\"rationale\":\"Requires escalation\",\"mitigation\":\"Block request\"}".into(),
        }));
        let verdict =
            verdict_from_choice(choice, "openai", "gpt-test").expect("should parse JSON payload");
        assert_eq!(verdict.label, "malicious");
        assert_eq!(verdict.mitigation, "Block request");
    }

    #[test]
    fn verdict_from_choice_empty_returns_fallback() {
        let choice = OneOrMany::one(AssistantContent::Text(Text { text: "".into() }));
        let verdict = verdict_from_choice(choice, "anthropic", "claude").expect("fallback verdict");
        assert_eq!(verdict.label, "unknown");
        assert!(verdict.rationale.contains("anthropic"));
    }
}
