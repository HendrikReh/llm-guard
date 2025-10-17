use super::{LlmClient, LlmSettings, ProviderKind};
use crate::scanner::{LlmVerdict, ScanReport};
use anyhow::{bail, Result};
use async_trait::async_trait;

/// Placeholder implementation that will be replaced with real rig.rs integration.
#[derive(Debug, Default)]
pub struct RigLlmClient;

impl RigLlmClient {
    pub fn from_settings(settings: &LlmSettings) -> Result<Box<dyn LlmClient>> {
        match settings.provider_kind()? {
            ProviderKind::Rig => bail!("rig.rs provider support not implemented yet"),
            _ => bail!("RigLlmClient expects provider kind 'rig'"),
        }
    }
}

#[async_trait]
impl LlmClient for RigLlmClient {
    async fn enrich(&self, _input: &str, _report: &ScanReport) -> Result<LlmVerdict> {
        bail!("RigLlmClient enrich not implemented")
    }
}
