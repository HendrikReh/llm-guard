use super::{LlmClient, LlmSettings, ProviderKind};
use crate::scanner::{LlmVerdict, ScanReport};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use rig::client::CompletionClient;
use rig::completion::message::AssistantContent;
use rig::completion::CompletionModelDyn;
use rig::providers::azure::AzureOpenAIAuth;
use rig::providers::{anthropic, azure, gemini, openai};
use serde::Deserialize;

const DEFAULT_OPENAI_MODEL: &str = "gpt-4o-mini";
const DEFAULT_ANTHROPIC_MODEL: &str = "claude-3-5-sonnet-latest";
const DEFAULT_GEMINI_MODEL: &str = "gemini-1.5-pro";
const MAX_OUTPUT_TOKENS: u64 = 200;
const TEMPERATURE: f64 = 0.1;
const SYSTEM_PROMPT: &str = "You are an application security assistant. Analyze prompt-injection scan results and respond with strict JSON: {\"label\": \"safe|suspicious|malicious\", \"rationale\": \"...\", \"mitigation\": \"...\"}. The mitigation should advise remediation steps.";

struct RigCompletionConfig {
    provider_label: &'static str,
    temperature: f64,
    max_tokens: u64,
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
            ProviderKind::Gemini => Ok(Box::new(Self::new_gemini(settings)?)),
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

        Ok(Self::from_model(model, "openai", model_id))
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

        Ok(Self::from_model(model, "anthropic", model_id))
    }

    fn new_gemini(settings: &LlmSettings) -> Result<Self> {
        if settings.api_key.trim().is_empty() {
            bail!("Gemini API key must be provided via LLM_GUARD_API_KEY");
        }

        let mut builder = gemini::Client::builder(&settings.api_key);
        if let Some(endpoint) = settings.endpoint.as_deref() {
            builder = builder.base_url(endpoint);
        }
        let client = builder
            .build()
            .context("failed to build gemini rig client")?;

        let model_id = settings
            .model
            .clone()
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_GEMINI_MODEL.to_string());

        let model: Box<dyn CompletionModelDyn + Send + Sync> =
            Box::new(client.completion_model(&model_id));

        Ok(Self::from_model(model, "gemini", model_id))
    }

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
            .or_else(|| settings.model.as_ref())
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

        Ok(Self::from_model(model, "azure", deployment))
    }

    fn from_model(
        model: Box<dyn CompletionModelDyn + Send + Sync>,
        provider_label: &'static str,
        model_id: String,
    ) -> Self {
        Self {
            model,
            config: RigCompletionConfig {
                provider_label,
                temperature: TEMPERATURE,
                max_tokens: MAX_OUTPUT_TOKENS,
            },
            model_id,
        }
    }
}

#[async_trait]
impl LlmClient for RigLlmClient {
    async fn enrich(&self, input: &str, report: &ScanReport) -> Result<LlmVerdict> {
        let prompt = format!(
            "Input excerpt:\n{}\n\nScore: {:.1} ({:?})\nTop findings: {}\n",
            truncate(input, 2000),
            report.risk_score,
            report.risk_band,
            serde_json::to_string(&report.findings).unwrap_or_default()
        );

        let request = self
            .model
            .completion_request(prompt.into())
            .preamble(SYSTEM_PROMPT.to_string())
            .temperature(self.config.temperature)
            .max_tokens(self.config.max_tokens)
            .build();

        let response = self.model.completion(request).await.with_context(|| {
            format!(
                "rig {} completion request failed for model {}",
                self.config.provider_label, self.model_id
            )
        })?;

        let content = response
            .choice
            .into_iter()
            .filter_map(|segment| match segment {
                AssistantContent::Text(text) => Some(text.text),
                AssistantContent::Reasoning(reasoning) => Some(reasoning.reasoning.join("\n")),
                AssistantContent::ToolCall(_) => None,
            })
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        if content.trim().is_empty() {
            bail!(
                "rig {} response did not return textual content",
                self.config.provider_label
            );
        }

        let trimmed = content.trim();
        let verdict: ModelVerdict = serde_json::from_str(trimmed).with_context(|| {
            format!(
                "rig {} response from model {} was not valid JSON verdict: {}",
                self.config.provider_label, self.model_id, trimmed
            )
        })?;

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

fn truncate(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    input.chars().take(max_chars).collect::<String>() + "…"
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
