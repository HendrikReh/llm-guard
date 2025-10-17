use anyhow::{Context, Result};
use std::collections::HashMap;

/// Environment-driven configuration required for LLM adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmSettings {
    pub provider: String,
    pub api_key: String,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub deployment: Option<String>,
    pub project: Option<String>,
    pub workspace: Option<String>,
    pub timeout_secs: Option<u64>,
    pub max_retries: u32,
    pub api_version: Option<String>,
}

impl LlmSettings {
    const PROVIDER_ENV: &'static str = "LLM_GUARD_PROVIDER";
    const API_KEY_ENV: &'static str = "LLM_GUARD_API_KEY";
    const ENDPOINT_ENV: &'static str = "LLM_GUARD_ENDPOINT";
    const MODEL_ENV: &'static str = "LLM_GUARD_MODEL";
    const DEPLOYMENT_ENV: &'static str = "LLM_GUARD_DEPLOYMENT";
    const PROJECT_ENV: &'static str = "LLM_GUARD_PROJECT";
    const WORKSPACE_ENV: &'static str = "LLM_GUARD_WORKSPACE";
    const TIMEOUT_ENV: &'static str = "LLM_GUARD_TIMEOUT_SECS";
    const RETRIES_ENV: &'static str = "LLM_GUARD_MAX_RETRIES";
    const API_VERSION_ENV: &'static str = "LLM_GUARD_API_VERSION";

    pub fn provider_kind(&self) -> Result<super::ProviderKind> {
        super::ProviderKind::from_provider(&self.provider)
    }

    /// Load settings from environment variables.
    ///
    /// * `LLM_GUARD_PROVIDER` — Provider identifier (default: `openai`).
    /// * `LLM_GUARD_API_KEY`  — API key/token (required).
    /// * `LLM_GUARD_ENDPOINT` — Optional custom endpoint/base URL.
    pub fn from_env() -> Result<Self> {
        Self::from_map(std::env::vars().collect())
    }

    fn from_map(vars: HashMap<String, String>) -> Result<Self> {
        let get_trimmed = |key: &str| -> Option<String> {
            vars.get(key)
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
                .map(|v| v.to_string())
        };
        let provider = vars
            .get(Self::PROVIDER_ENV)
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "openai".to_string());
        let provider_lower = provider.to_lowercase();
        let api_key = match provider_lower.as_str() {
            "noop" => get_trimmed(Self::API_KEY_ENV).unwrap_or_default(),
            _ => vars
                .get(Self::API_KEY_ENV)
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
                .map(|v| v.to_string())
                .with_context(|| {
                    format!(
                        "environment variable {} must be set when --with-llm is used",
                        Self::API_KEY_ENV
                    )
                })?,
        };
        let endpoint = get_trimmed(Self::ENDPOINT_ENV);
        let model = get_trimmed(Self::MODEL_ENV);
        let deployment = get_trimmed(Self::DEPLOYMENT_ENV);
        let project = get_trimmed(Self::PROJECT_ENV);
        let workspace = get_trimmed(Self::WORKSPACE_ENV);
        let timeout_secs = vars
            .get(Self::TIMEOUT_ENV)
            .and_then(|v| v.trim().parse::<u64>().ok());
        let max_retries = vars
            .get(Self::RETRIES_ENV)
            .and_then(|v| v.trim().parse::<u32>().ok())
            .unwrap_or(2);
        let api_version = get_trimmed(Self::API_VERSION_ENV);

        Ok(Self {
            provider,
            api_key,
            endpoint,
            model,
            deployment,
            project,
            workspace,
            timeout_secs,
            max_retries,
            api_version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use proptest::prelude::*;
    use std::collections::HashMap;
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
            env::remove_var(LlmSettings::TIMEOUT_ENV);
            env::remove_var(LlmSettings::RETRIES_ENV);
            env::remove_var(LlmSettings::API_VERSION_ENV);

            let settings = LlmSettings::from_env().expect("should load settings");
            assert_eq!(settings.provider, "openai");
            assert_eq!(settings.api_key, "secret");
            assert!(settings.endpoint.is_none());
            assert!(settings.model.is_none());
            assert!(settings.deployment.is_none());
            assert!(settings.project.is_none());
            assert!(settings.workspace.is_none());
            assert_eq!(settings.max_retries, 2);
            assert!(settings.api_version.is_none());
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
            env::remove_var(LlmSettings::TIMEOUT_ENV);
            env::remove_var(LlmSettings::RETRIES_ENV);
            env::remove_var(LlmSettings::API_VERSION_ENV);
            env::remove_var(LlmSettings::DEPLOYMENT_ENV);
            env::remove_var(LlmSettings::PROJECT_ENV);
            env::remove_var(LlmSettings::WORKSPACE_ENV);
            let settings = LlmSettings::from_env().expect("noop should not require key");
            assert_eq!(settings.provider, "noop");
            assert!(settings.api_key.is_empty());
            assert!(settings.deployment.is_none());
            assert!(settings.project.is_none());
            assert!(settings.workspace.is_none());
        });
    }

    #[test]
    fn parses_timeout_and_retries() {
        with_env_lock(|| {
            env::set_var(LlmSettings::PROVIDER_ENV, "openai");
            env::set_var(LlmSettings::API_KEY_ENV, "secret");
            env::set_var(LlmSettings::TIMEOUT_ENV, "45");
            env::set_var(LlmSettings::RETRIES_ENV, "5");
            env::set_var(LlmSettings::API_VERSION_ENV, "2024-07-01");
            env::set_var(LlmSettings::DEPLOYMENT_ENV, "deployment");
            env::set_var(LlmSettings::PROJECT_ENV, "project");
            env::set_var(LlmSettings::WORKSPACE_ENV, "workspace");
            let settings = LlmSettings::from_env().expect("should parse timeout/retries");
            assert_eq!(settings.timeout_secs, Some(45));
            assert_eq!(settings.max_retries, 5);
            assert_eq!(settings.api_version.as_deref(), Some("2024-07-01"));
            assert_eq!(settings.deployment.as_deref(), Some("deployment"));
            assert_eq!(settings.project.as_deref(), Some("project"));
            assert_eq!(settings.workspace.as_deref(), Some("workspace"));
            env::remove_var(LlmSettings::TIMEOUT_ENV);
            env::remove_var(LlmSettings::RETRIES_ENV);
            env::remove_var(LlmSettings::API_VERSION_ENV);
            env::remove_var(LlmSettings::DEPLOYMENT_ENV);
            env::remove_var(LlmSettings::PROJECT_ENV);
            env::remove_var(LlmSettings::WORKSPACE_ENV);
        });
    }

    fn trimmed_string() -> impl Strategy<Value = String> {
        proptest::string::string_regex("[A-Za-z0-9 _\\-]{1,24}").unwrap()
    }

    proptest! {
        #[test]
        fn from_map_trims_values_and_defaults(
            provider in prop_oneof![
                Just("openai".to_string()),
                Just("anthropic".to_string()),
                Just("gemini".to_string()),
                Just("noop".to_string()),
            ],
            api_key in proptest::option::of(trimmed_string()),
            endpoint in proptest::option::of(trimmed_string()),
            model in proptest::option::of(trimmed_string()),
            timeout in proptest::option::of(0u64..120u64),
            retries in proptest::option::of(0u32..6u32)
        ) {
            let mut vars = HashMap::new();
            vars.insert(
                LlmSettings::PROVIDER_ENV.to_string(),
                format!("  {}  ", provider)
            );

            match provider.as_str() {
                "noop" => {
                    if let Some(key) = api_key.clone() {
                        vars.insert(LlmSettings::API_KEY_ENV.to_string(), format!("  {}  ", key));
                    }
                }
                _ => {
                    let key = api_key.clone().unwrap_or_else(|| "secret-key".to_string());
                    vars.insert(LlmSettings::API_KEY_ENV.to_string(), format!("  {}  ", key));
                }
            }

            if let Some(ep) = endpoint.clone() {
                vars.insert(LlmSettings::ENDPOINT_ENV.to_string(), format!("  {}  ", ep));
            }
            if let Some(model) = model.clone() {
                vars.insert(LlmSettings::MODEL_ENV.to_string(), format!("  {}  ", model));
            }
            if let Some(t) = timeout {
                vars.insert(LlmSettings::TIMEOUT_ENV.to_string(), format!("  {}  ", t));
            }
            if let Some(r) = retries {
                vars.insert(LlmSettings::RETRIES_ENV.to_string(), format!("  {}  ", r));
            }

            let settings = LlmSettings::from_map(vars).expect("settings should parse");
            prop_assert_eq!(settings.provider, provider.trim());
            if provider == "noop" {
                if let Some(key) = api_key.clone() {
                    prop_assert_eq!(settings.api_key, key.trim());
                } else {
                    prop_assert!(settings.api_key.is_empty());
                }
            } else {
                let expected_key = api_key.unwrap_or_else(|| "secret-key".to_string());
                prop_assert_eq!(settings.api_key, expected_key.trim());
            }
            if let Some(ep) = endpoint {
                prop_assert_eq!(settings.endpoint.as_deref(), Some(ep.trim()));
            } else {
                prop_assert!(settings.endpoint.is_none());
            }
            if let Some(model) = model {
                prop_assert_eq!(settings.model.as_deref(), Some(model.trim()));
            }
            match timeout {
                Some(t) => prop_assert_eq!(settings.timeout_secs, Some(t)),
                None => prop_assert!(settings.timeout_secs.is_none()),
            }
            let expected_retries = retries.unwrap_or(2);
            prop_assert_eq!(settings.max_retries, expected_retries);
        }
    }
}
