use super::{LlmClient, LlmSettings};
use crate::scanner::{LlmVerdict, ScanReport};
use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct GeminiClient {
    http: Client,
    url: String,
    api_key: String,
    max_retries: u32,
}

impl GeminiClient {
    pub fn new(settings: &LlmSettings) -> Result<Self> {
        if settings.api_key.trim().is_empty() {
            bail!("Gemini API key must be provided via LLM_GUARD_API_KEY");
        }
        let base = settings
            .endpoint
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com".to_string());
        let model = settings
            .model
            .clone()
            .unwrap_or_else(|| "gemini-1.5-flash".to_string());
        let url = format!(
            "{}/v1beta/models/{}:generateContent",
            base.trim_end_matches('/'),
            model
        );
        let http = Client::builder()
            .user_agent("llm-guard/0.1")
            .timeout(Duration::from_secs(settings.timeout_secs.unwrap_or(30)))
            .build()
            .context("failed to build Gemini HTTP client")?;
        Ok(Self {
            http,
            url,
            api_key: settings.api_key.clone(),
            max_retries: settings.max_retries,
        })
    }
}

#[async_trait]
impl LlmClient for GeminiClient {
    async fn enrich(&self, input: &str, report: &ScanReport) -> Result<LlmVerdict> {
        let payload = GeminiRequest {
            contents: vec![GeminiRequestContent {
                role: "user".into(),
                parts: vec![GeminiRequestPart {
                    text: Some(format!(
                        "{}\n\nInput excerpt:\n{}\n\nScore: {:.1} ({:?})\nTop findings: {}\n",
                        SYSTEM_PROMPT,
                        truncate(input, 2000),
                        report.risk_score,
                        report.risk_band,
                        serde_json::to_string(&report.findings).unwrap_or_default()
                    )),
                }],
            }],
        };

        let mut attempt = 0u32;
        let mut backoff = Duration::from_millis(200);
        loop {
            let response = self
                .http
                .post(&self.url)
                .query(&[("key", &self.api_key)])
                .json(&payload)
                .send()
                .await;

            let response = match response {
                Ok(resp) => resp,
                Err(err) => {
                    if attempt >= self.max_retries {
                        return Err(err).context("failed to call Gemini generateContent API");
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
                    bail!("Gemini API error ({}): {}", status, body);
                }
                sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(5));
                attempt += 1;
                continue;
            }

            let message: GeminiResponse = response
                .json()
                .await
                .context("failed to parse Gemini response")?;
            let content = message
                .candidates
                .into_iter()
                .flat_map(|candidate| candidate.content.parts)
                .filter_map(|part| part.text)
                .next()
                .ok_or_else(|| anyhow!("Gemini response missing message content"))?;

            let verdict: ModelVerdict = serde_json::from_str(&content)
                .context("expected JSON verdict from Gemini response")?;

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
struct GeminiRequest {
    contents: Vec<GeminiRequestContent>,
}

#[derive(Serialize)]
struct GeminiRequestContent {
    role: String,
    parts: Vec<GeminiRequestPart>,
}

#[derive(Serialize)]
struct GeminiRequestPart {
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
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
    use serde_json::json;

    fn base_settings(url: String) -> LlmSettings {
        LlmSettings {
            provider: "gemini".into(),
            api_key: "test-key".into(),
            endpoint: Some(url),
            model: Some("gemini-test".into()),
            timeout_secs: Some(5),
            max_retries: 0,
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
                .path("/v1beta/models/gemini-test:generateContent")
                .query_param("key", "test-key");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "candidates": [
                        {
                            "content": {
                                "role": "model",
                                "parts": [
                                    {"text": "{\"label\":\"safe\",\"rationale\":\"ok\",\"mitigation\":\"none\"}"}
                                ]
                            }
                        }
                    ]
                }));
        });

        let client = GeminiClient::new(&base_settings(server.base_url())).unwrap();
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
            when.method(POST)
                .path("/v1beta/models/gemini-test:generateContent")
                .query_param("key", "test-key");
            then.status(500);
        });

        let mut settings = base_settings(server.base_url());
        settings.max_retries = 1;
        let client = GeminiClient::new(&settings).unwrap();
        let err = client.enrich("hello", &empty_report()).await.unwrap_err();
        assert!(err.to_string().contains("Gemini API error"));
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
