use super::{LlmClient, LlmSettings};
use crate::scanner::{LlmVerdict, ScanReport};
use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct AnthropicClient {
    http: Client,
    url: String,
    api_key: String,
    model: String,
    max_retries: u32,
}

impl AnthropicClient {
    pub fn new(settings: &LlmSettings) -> Result<Self> {
        if settings.api_key.trim().is_empty() {
            bail!("Anthropic API key must be provided via LLM_GUARD_API_KEY");
        }
        let base = settings
            .endpoint
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());
        let url = format!("{}/v1/messages", base.trim_end_matches('/'));
        let http = Client::builder()
            .user_agent("llm-guard/0.1")
            .timeout(Duration::from_secs(settings.timeout_secs.unwrap_or(30)))
            .build()
            .context("failed to build Anthropic HTTP client")?;
        Ok(Self {
            http,
            url,
            api_key: settings.api_key.clone(),
            model: settings
                .model
                .clone()
                .unwrap_or_else(|| "claude-3-haiku-20240307".to_string()),
            max_retries: settings.max_retries,
        })
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn enrich(&self, input: &str, report: &ScanReport) -> Result<LlmVerdict> {
        let payload = AnthropicRequest {
            model: self.model.clone(),
            system: SYSTEM_PROMPT.to_string(),
            messages: vec![AnthropicMessage {
                role: "user".into(),
                content: format!(
                    "Input excerpt:\n{}\n\nScore: {:.1} ({:?})\nTop findings: {}\n",
                    truncate(input, 2000),
                    report.risk_score,
                    report.risk_band,
                    serde_json::to_string(&report.findings).unwrap_or_default()
                ),
            }],
            max_tokens: 200,
        };

        let mut attempt = 0u32;
        let mut backoff = Duration::from_millis(200);
        loop {
            let response = self
                .http
                .post(&self.url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&payload)
                .send()
                .await;

            let response = match response {
                Ok(resp) => resp,
                Err(err) => {
                    if attempt >= self.max_retries {
                        return Err(err).context("failed to call Anthropic messages API");
                    }
                    sleep(backoff).await;
                    backoff = (backoff * 2).min(Duration::from_secs(5));
                    attempt += 1;
                    continue;
                }
            };

            if !response.status().is_success() {
                if attempt >= self.max_retries {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    bail!("Anthropic API error ({}): {}", status, body);
                }
                sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(5));
                attempt += 1;
                continue;
            }

            let message: AnthropicResponse = response
                .json()
                .await
                .context("failed to parse Anthropic response")?;
            let content = message
                .content
                .into_iter()
                .find_map(|part| part.text)
                .ok_or_else(|| anyhow!("Anthropic response missing message content"))?;

            let verdict: ModelVerdict = serde_json::from_str(&content)
                .context("expected JSON verdict from Anthropic response")?;

            return Ok(LlmVerdict {
                label: verdict.label,
                rationale: verdict.rationale,
                mitigation: verdict.mitigation,
            });
        }
    }
}

const SYSTEM_PROMPT: &str = "You are an application security assistant. Analyze prompt-injection scan results and respond with strict JSON: {\"label\": \"safe|suspicious|malicious\", \"rationale\": \"...\", \"mitigation\": \"...\"}. The mitigation should advise remediation steps.";

fn truncate(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    input.chars().take(max_chars).collect::<String>() + "…"
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    system: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    _type: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
struct ModelVerdict {
    label: String,
    rationale: String,
    mitigation: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::settings::LlmSettings;
    use crate::scanner::{RiskThresholds, ScanReport, ScoreBreakdown};
    use httpmock::prelude::*;

    fn base_settings(url: String) -> LlmSettings {
        LlmSettings {
            provider: "anthropic".into(),
            api_key: "test-key".into(),
            endpoint: Some(url),
            model: Some("claude-test".into()),
            deployment: None,
            project: None,
            workspace: None,
            timeout_secs: Some(5),
            max_retries: 0,
            api_version: None,
        }
    }

    fn empty_report() -> ScanReport {
        ScanReport::from_breakdown(
            vec![],
            0,
            None,
            ScoreBreakdown::default(),
            &RiskThresholds::default(),
        )
    }

    #[tokio::test]
    #[ignore = "requires loopback networking"]
    async fn enrich_parses_successful_response() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v1/messages")
                .header("x-api-key", "test-key");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"content":[{"type":"text","text":"{\"label\":\"safe\",\"rationale\":\"ok\",\"mitigation\":\"none\"}"}]}"#);
        });

        let client = AnthropicClient::new(&base_settings(server.base_url())).unwrap();
        let verdict = client.enrich("hello", &empty_report()).await.unwrap();
        assert_eq!(verdict.label, "safe");
        assert_eq!(verdict.rationale, "ok");
        assert_eq!(verdict.mitigation, "none");
        mock.assert();
    }

    #[tokio::test]
    #[ignore = "requires loopback networking"]
    async fn retries_on_failure() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/v1/messages");
            then.status(500);
        });

        let mut settings = base_settings(server.base_url());
        settings.max_retries = 1;
        let client = AnthropicClient::new(&settings).unwrap();
        let err = client.enrich("hello", &empty_report()).await.unwrap_err();
        assert!(err.to_string().contains("Anthropic API error"));
        mock.assert_hits(2);
    }

    #[test]
    fn truncate_short_strings_return_same() {
        assert_eq!(truncate("abc", 10), "abc");
    }

    #[test]
    fn truncate_long_strings_adds_ellipsis() {
        let result = truncate("abcdefghijklmnopqrstuvwxyz", 5);
        assert_eq!(result, "abcde…");
    }
}
