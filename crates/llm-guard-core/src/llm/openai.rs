use super::{LlmClient, LlmSettings};
use crate::scanner::{LlmVerdict, ScanReport};
use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct OpenAiClient {
    http: Client,
    url: String,
    api_key: String,
    model: String,
}

impl OpenAiClient {
    pub fn new(settings: &LlmSettings) -> Result<Self> {
        if settings.api_key.trim().is_empty() {
            bail!("OpenAI API key must be provided via LLM_GUARD_API_KEY");
        }
        let base = settings
            .endpoint
            .clone()
            .unwrap_or_else(|| "https://api.openai.com".to_string());
        let url = format!("{}/v1/chat/completions", base.trim_end_matches('/'));
        let http = Client::builder()
            .user_agent("llm-guard/0.1")
            .build()
            .context("failed to build OpenAI HTTP client")?;
        Ok(Self {
            http,
            url,
            api_key: settings.api_key.clone(),
            model: settings
                .model
                .clone()
                .unwrap_or_else(|| "gpt-4o-mini".to_string()),
        })
    }
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn enrich(&self, input: &str, report: &ScanReport) -> Result<LlmVerdict> {
        let payload = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: SYSTEM_PROMPT.to_string(),
                },
                ChatMessage {
                    role: "user",
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

        let response = self
            .http
            .post(&self.url)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .context("failed to call OpenAI chat completions API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("OpenAI API error ({}): {}", status, body);
        }

        let chat: ChatCompletionResponse = response
            .json()
            .await
            .context("failed to parse OpenAI response")?;
        let content = chat
            .choices
            .into_iter()
            .find_map(|choice| choice.message.content)
            .ok_or_else(|| anyhow!("OpenAI response missing message content"))?;

        let verdict: ModelVerdict =
            serde_json::from_str(&content).context("expected JSON verdict from OpenAI response")?;

        Ok(LlmVerdict {
            label: verdict.label,
            rationale: verdict.rationale,
            mitigation: verdict.mitigation,
        })
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
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
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
