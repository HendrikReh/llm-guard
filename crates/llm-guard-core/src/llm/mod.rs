mod anthropic;
mod azure;
mod gemini;
mod openai;
mod rig_adapter;
mod settings;

use anyhow::{bail, Result};
use async_trait::async_trait;

use crate::scanner::{LlmVerdict, ScanReport};

pub use anthropic::AnthropicClient;
pub use azure::AzureOpenAiClient;
pub use gemini::GeminiClient;
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
    match ProviderKind::from_provider(settings.provider.trim())? {
        ProviderKind::Noop => Ok(Box::new(NoopLlmClient::default())),
        ProviderKind::OpenAi => Ok(Box::new(OpenAiClient::new(settings)?)),
        ProviderKind::Azure => Ok(Box::new(AzureOpenAiClient::new(settings)?)),
        ProviderKind::Anthropic => Ok(Box::new(AnthropicClient::new(settings)?)),
        ProviderKind::Gemini => Ok(Box::new(GeminiClient::new(settings)?)),
        ProviderKind::Rig => rig_adapter::RigLlmClient::from_settings(settings),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Noop,
    OpenAi,
    Azure,
    Anthropic,
    Gemini,
    Rig,
}

impl ProviderKind {
    pub fn from_provider(name: &str) -> Result<Self> {
        match name.to_ascii_lowercase().as_str() {
            "noop" => Ok(ProviderKind::Noop),
            "openai" | "open-ai" => Ok(ProviderKind::OpenAi),
            "azure" | "azure-openai" => Ok(ProviderKind::Azure),
            "anthropic" | "claude" => Ok(ProviderKind::Anthropic),
            "gemini" | "google" | "google-gemini" => Ok(ProviderKind::Gemini),
            "rig" | "rag" => Ok(ProviderKind::Rig),
            other => bail!("unsupported LLM provider `{}`", other),
        }
    }
}
