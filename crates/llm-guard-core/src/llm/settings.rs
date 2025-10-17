use anyhow::{Context, Result};
use std::collections::HashMap;

/// Environment-driven configuration required for LLM adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmSettings {
    pub provider: String,
    pub api_key: String,
    pub endpoint: Option<String>,
    pub model: Option<String>,
}

impl LlmSettings {
    const PROVIDER_ENV: &'static str = "LLM_GUARD_PROVIDER";
    const API_KEY_ENV: &'static str = "LLM_GUARD_API_KEY";
    const ENDPOINT_ENV: &'static str = "LLM_GUARD_ENDPOINT";
    const MODEL_ENV: &'static str = "LLM_GUARD_MODEL";

    /// Load settings from environment variables.
    ///
    /// * `LLM_GUARD_PROVIDER` — Provider identifier (default: `openai`).
    /// * `LLM_GUARD_API_KEY`  — API key/token (required).
    /// * `LLM_GUARD_ENDPOINT` — Optional custom endpoint/base URL.
    pub fn from_env() -> Result<Self> {
        Self::from_map(std::env::vars().collect())
    }

    fn from_map(vars: HashMap<String, String>) -> Result<Self> {
        let provider = vars
            .get(Self::PROVIDER_ENV)
            .cloned()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "openai".to_string())
            .trim()
            .to_string();
        let provider_lower = provider.to_lowercase();
        let api_key = match provider_lower.as_str() {
            "noop" => vars.get(Self::API_KEY_ENV).cloned().unwrap_or_default(),
            _ => vars
                .get(Self::API_KEY_ENV)
                .cloned()
                .filter(|v| !v.trim().is_empty())
                .with_context(|| {
                    format!(
                        "environment variable {} must be set when --with-llm is used",
                        Self::API_KEY_ENV
                    )
                })?,
        };
        let endpoint = vars
            .get(Self::ENDPOINT_ENV)
            .cloned()
            .filter(|v| !v.trim().is_empty());
        let model = vars
            .get(Self::MODEL_ENV)
            .cloned()
            .filter(|v| !v.trim().is_empty());

        Ok(Self {
            provider,
            api_key,
            endpoint,
            model,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::env;
    use std::sync::Mutex;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn with_env_lock<F: FnOnce()>(func: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        func();
    }

    #[test]
    fn defaults_to_openai_provider() {
        with_env_lock(|| {
            env::remove_var(LlmSettings::PROVIDER_ENV);
            env::set_var(LlmSettings::API_KEY_ENV, "secret");
            env::remove_var(LlmSettings::ENDPOINT_ENV);
            env::remove_var(LlmSettings::MODEL_ENV);

            let settings = LlmSettings::from_env().expect("should load settings");
            assert_eq!(settings.provider, "openai");
            assert_eq!(settings.api_key, "secret");
            assert!(settings.endpoint.is_none());
            assert!(settings.model.is_none());
        });
    }

    #[test]
    fn errors_when_api_key_missing() {
        with_env_lock(|| {
            env::set_var(LlmSettings::PROVIDER_ENV, "openai");
            env::remove_var(LlmSettings::API_KEY_ENV);
            let err = LlmSettings::from_env().expect_err("missing API key should error");
            assert!(err.to_string().contains(LlmSettings::API_KEY_ENV));
        });
    }

    #[test]
    fn noop_provider_allows_missing_key() {
        with_env_lock(|| {
            env::set_var(LlmSettings::PROVIDER_ENV, "noop");
            env::remove_var(LlmSettings::API_KEY_ENV);
            let settings = LlmSettings::from_env().expect("noop should not require key");
            assert_eq!(settings.provider, "noop");
            assert!(settings.api_key.is_empty());
        });
    }
}
