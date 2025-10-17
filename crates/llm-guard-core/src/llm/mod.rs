mod anthropic;
mod openai;
mod settings;

use anyhow::{bail, Result};
use async_trait::async_trait;

use crate::scanner::{LlmVerdict, ScanReport};

pub use anthropic::AnthropicClient;
pub use openai::OpenAiClient;
pub use settings::LlmSettings;

/// Client abstraction for invoking large language models to enrich scan results.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Produce a verdict/rationale given the original input and heuristic scan report.
    async fn enrich(&self, input: &str, report: &ScanReport) -> Result<LlmVerdict>;
}

/// Placeholder implementation used until a concrete adapter is wired in.
#[derive(Debug, Default, Clone)]
pub struct NoopLlmClient;

#[async_trait]
impl LlmClient for NoopLlmClient {
    async fn enrich(&self, _input: &str, _report: &ScanReport) -> Result<LlmVerdict> {
        Ok(LlmVerdict {
            label: "unavailable".into(),
            rationale: "LLM adapter not configured; returning heuristic-only verdict.".into(),
            mitigation: "Configure Phase 6 LLM client to receive enriched guidance.".into(),
        })
    }
}

pub fn build_client(settings: &LlmSettings) -> Result<Box<dyn LlmClient>> {
    match settings.provider.to_lowercase().as_str() {
        "noop" => Ok(Box::new(NoopLlmClient::default())),
        "openai" | "open-ai" => Ok(Box::new(OpenAiClient::new(settings)?)),
        "anthropic" | "claude" => Ok(Box::new(AnthropicClient::new(settings)?)),
        other => bail!("unsupported LLM provider `{}`", other),
    }
}
