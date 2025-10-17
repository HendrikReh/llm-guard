use super::{LlmClient, LlmSettings};
use crate::scanner::{LlmVerdict, ScanReport};
use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct AzureOpenAiClient {
    http: Client,
    url: String,
    api_key: String,
    max_retries: u32,
}

impl AzureOpenAiClient {
    pub fn new(settings: &LlmSettings) -> Result<Self> {
        if settings.api_key.trim().is_empty() {
            bail!("Azure OpenAI API key must be provided via LLM_GUARD_API_KEY");
        }
        let endpoint = settings
            .endpoint
            .clone()
            .ok_or_else(|| anyhow!("LLM_GUARD_ENDPOINT must be set for azure provider"))?;
        let deployment = settings.model.clone().ok_or_else(|| {
            anyhow!("LLM_GUARD_MODEL must contain the deployment name for azure provider")
        })?;
        let api_version = settings
            .api_version
            .clone()
            .unwrap_or_else(|| "2024-02-15-preview".to_string());

        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            endpoint.trim_end_matches('/'),
            deployment,
            api_version
        );

        let http = Client::builder()
            .user_agent("llm-guard/0.1")
            .timeout(Duration::from_secs(settings.timeout_secs.unwrap_or(30)))
            .build()
            .context("failed to build Azure OpenAI HTTP client")?;

        Ok(Self {
            http,
            url,
            api_key: settings.api_key.clone(),
            max_retries: settings.max_retries,
        })
    }
}

#[async_trait]
impl LlmClient for AzureOpenAiClient {
    async fn enrich(&self, input: &str, report: &ScanReport) -> Result<LlmVerdict> {
        let payload = ChatCompletionRequest {
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: SYSTEM_PROMPT.into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: format!(
                        "Input excerpt:\n{}\n\nScore: {:.1} ({:?})\nTop findings: {}\n",
                        truncate(input, 2000),
                        report.risk_score,
                        report.risk_band,
                        serde_json::to_string(&report.findings).unwrap_or_default()
                    ),
                },
            ],
            temperature: 0.1,
            max_tokens: 200,
        };

        let mut attempt = 0u32;
        let mut backoff = Duration::from_millis(200);
        loop {
            let response = self
                .http
                .post(&self.url)
                .header("api-key", &self.api_key)
                .json(&payload)
                .send()
                .await;

            let response = match response {
                Ok(resp) => resp,
                Err(err) => {
                    if attempt >= self.max_retries {
                        return Err(err)
                            .context("failed to call Azure OpenAI chat completions API");
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
                    bail!("Azure OpenAI API error ({}): {}", status, body);
                }
                sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(5));
                attempt += 1;
                continue;
            }

            let chat: ChatCompletionResponse = response
                .json()
                .await
                .context("failed to parse Azure OpenAI response")?;
            let content = chat
                .choices
                .into_iter()
                .find_map(|choice| choice.message.content)
                .ok_or_else(|| anyhow!("Azure OpenAI response missing message content"))?;

            let verdict: ModelVerdict = serde_json::from_str(&content)
                .context("expected JSON verdict from Azure OpenAI response")?;

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
struct ChatCompletionRequest {
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
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
    use crate::scanner::{RiskThresholds, ScanReport, ScoreBreakdown};
    use httpmock::prelude::*;

    fn base_settings(url: String) -> LlmSettings {
        LlmSettings {
            provider: "azure".into(),
            api_key: "test-key".into(),
            endpoint: Some(url),
            model: Some("deployment-name".into()),
            timeout_secs: Some(5),
            max_retries: 0,
            api_version: Some("2024-02-15-preview".into()),
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
                .path("/openai/deployments/deployment-name/chat/completions")
                .query_param("api-version", "2024-02-15-preview")
                .header("api-key", "test-key");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"choices":[{"message":{"content":"{\"label\":\"safe\",\"rationale\":\"ok\",\"mitigation\":\"none\"}"}}]}"#);
        });

        let client = AzureOpenAiClient::new(&base_settings(server.base_url())).unwrap();
        let verdict = client.enrich("hello", &empty_report()).await.unwrap();
        assert_eq!(verdict.label, "safe");
        mock.assert();
    }

    #[tokio::test]
    #[ignore = "requires loopback networking"]
    async fn retries_on_failure() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/openai/deployments/deployment-name/chat/completions")
                .query_param("api-version", "2024-02-15-preview");
            then.status(500);
        });

        let mut settings = base_settings(server.base_url());
        settings.max_retries = 1;
        let client = AzureOpenAiClient::new(&settings).unwrap();
        let err = client.enrich("hello", &empty_report()).await.unwrap_err();
        assert!(err.to_string().contains("Azure OpenAI API error"));
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
