use std::collections::HashMap;
use std::env;
use std::fs as stdfs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use llm_guard_core::{
    build_client, render_report, DefaultScanner, FileRuleRepository, LlmClient, LlmSettings,
    OutputFormat, RiskBand, RiskThresholds, RuleKind, RuleRepository, ScanReport, Scanner,
    ScoreBreakdown,
};
use serde::Deserialize;
use serde_yaml;
use tokio::{
    fs,
    io::{self, AsyncReadExt},
    signal,
    time::sleep,
};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "llm-guard",
    author,
    version,
    about = "Prompt Injection Firewall CLI"
)]
struct Cli {
    /// Directory containing rule packs (keywords.txt, patterns.json)
    #[arg(
        long = "rules-dir",
        value_name = "DIR",
        default_value = "./rules",
        global = true
    )]
    rules_dir: PathBuf,

    /// Optional configuration file providing defaults (TOML/YAML/JSON)
    #[arg(long = "config", value_name = "FILE", global = true)]
    config_file: Option<PathBuf>,

    /// Optional provider configuration file containing per-provider credentials.
    #[arg(
        long = "providers-config",
        value_name = "FILE",
        default_value = "llm_providers.yaml",
        global = true
    )]
    providers_config: PathBuf,

    /// Enable verbose diagnostics (including raw provider payloads on errors).
    #[arg(long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all loaded rules.
    ListRules {
        /// Emit rules as JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
    },
    /// Scan input (stdin or file) and produce a risk report.
    Scan {
        /// Optional path to a file to scan; omit to read from stdin.
        #[arg(long)]
        file: Option<PathBuf>,
        /// Emit JSON instead of human-readable output.
        #[arg(long)]
        json: bool,
        /// Tail the specified file for changes (requires --file).
        #[arg(long)]
        tail: bool,
        /// Augment heuristic report with LLM verdict (not yet implemented).
        #[arg(long = "with-llm")]
        with_llm: bool,
        /// Override provider (e.g., openai, anthropic, gemini, noop).
        #[arg(long)]
        provider: Option<String>,
        /// Override model identifier for the selected provider.
        #[arg(long)]
        model: Option<String>,
        /// Override endpoint/base URL for the selected provider.
        #[arg(long)]
        endpoint: Option<String>,
        /// Override deployment identifier (Azure) when using rig-backed providers.
        #[arg(long)]
        deployment: Option<String>,
        /// Override project identifier for providers that require it.
        #[arg(long)]
        project: Option<String>,
        /// Override workspace identifier for providers that require it.
        #[arg(long)]
        workspace: Option<String>,
    },
    /// Execute health checks against configured LLM providers.
    Health {
        /// Limit the health check to a single provider name.
        #[arg(long)]
        provider: Option<String>,
        /// Skip the live LLM call; only validate configuration/build steps.
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Deserialize, Clone)]
struct ProviderProfile {
    name: String,
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    endpoint: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    deployment: Option<String>,
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    workspace: Option<String>,
    #[serde(default)]
    timeout_secs: Option<u64>,
    #[serde(default)]
    max_retries: Option<u32>,
    #[serde(default)]
    api_version: Option<String>,
}

#[derive(Debug, Default)]
struct ProviderProfiles {
    entries: HashMap<String, ProviderProfile>,
}

impl ProviderProfiles {
    fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = stdfs::read_to_string(path).with_context(|| {
            format!(
                "failed to read provider configuration from {}",
                path.display()
            )
        })?;

        if contents.trim().is_empty() {
            return Ok(Self::default());
        }

        // Support either a top-level `providers` key or a bare list of profiles.
        #[derive(Deserialize)]
        struct ProviderConfigWrapper {
            providers: Vec<ProviderProfile>,
        }

        let profiles = match serde_yaml::from_str::<ProviderConfigWrapper>(&contents) {
            Ok(wrapper) => wrapper.providers,
            Err(_) => serde_yaml::from_str::<Vec<ProviderProfile>>(&contents)
                .with_context(|| "invalid provider configuration structure")?,
        };

        let mut entries = HashMap::new();
        for profile in profiles {
            entries.insert(profile.name.to_ascii_lowercase(), profile);
        }

        Ok(Self { entries })
    }

    fn prime_env(&self, provider: &str) {
        if let Some(profile) = self.get(provider) {
            maybe_set_env("LLM_GUARD_PROVIDER", Some(profile.name.clone()));
            maybe_set_env("LLM_GUARD_API_KEY", profile.api_key.clone());
            maybe_set_env("LLM_GUARD_ENDPOINT", profile.endpoint.clone());
            maybe_set_env("LLM_GUARD_MODEL", profile.model.clone());
            maybe_set_env("LLM_GUARD_DEPLOYMENT", profile.deployment.clone());
            maybe_set_env("LLM_GUARD_PROJECT", profile.project.clone());
            maybe_set_env("LLM_GUARD_WORKSPACE", profile.workspace.clone());
            maybe_set_env(
                "LLM_GUARD_TIMEOUT_SECS",
                profile.timeout_secs.map(|timeout| timeout.to_string()),
            );
            maybe_set_env(
                "LLM_GUARD_MAX_RETRIES",
                profile.max_retries.map(|retries| retries.to_string()),
            );
            maybe_set_env("LLM_GUARD_API_VERSION", profile.api_version.clone());
        }
    }

    fn apply_defaults(&self, provider: &str, settings: &mut LlmSettings) {
        if let Some(profile) = self.get(provider) {
            if settings.model.is_none() {
                settings.model = profile.model.clone();
            }
            if settings.deployment.is_none() {
                settings.deployment = profile.deployment.clone();
            }
            if settings.project.is_none() {
                settings.project = profile.project.clone();
            }
            if settings.workspace.is_none() {
                settings.workspace = profile.workspace.clone();
            }
            if settings.timeout_secs.is_none() && std::env::var("LLM_GUARD_TIMEOUT_SECS").is_err() {
                settings.timeout_secs = profile.timeout_secs;
            }
            if let Some(retries) = profile.max_retries {
                if std::env::var("LLM_GUARD_MAX_RETRIES").is_err() {
                    settings.max_retries = retries;
                }
            }
            if settings.api_version.is_none() && std::env::var("LLM_GUARD_API_VERSION").is_err() {
                settings.api_version = profile.api_version.clone();
            }
        }
    }

    fn get(&self, provider: &str) -> Option<&ProviderProfile> {
        self.entries.get(&provider.to_ascii_lowercase())
    }

    fn names(&self) -> Vec<String> {
        self.entries
            .values()
            .map(|profile| profile.name.clone())
            .collect()
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

struct EnvGuard {
    snapshot: Vec<(String, Option<String>)>,
}

impl EnvGuard {
    fn new() -> Self {
        Self {
            snapshot: Vec::new(),
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        if !self.snapshot.iter().any(|(k, _)| k == key) {
            self.snapshot.push((key.to_string(), env::var(key).ok()));
        }
        env::set_var(key, value);
    }

    fn maybe_set(&mut self, key: &str, value: Option<&str>) {
        if let Some(val) = value {
            self.set(key, val);
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, previous) in self.snapshot.drain(..).rev() {
            if let Some(value) = previous {
                env::set_var(&key, value);
            } else {
                env::remove_var(&key);
            }
        }
    }
}

#[cfg(test)]
mod provider_config_tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::collections::HashMap;
    use std::env;
    use std::sync::Mutex;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn reset_vars() {
        env::remove_var("LLM_GUARD_PROVIDER");
        env::remove_var("LLM_GUARD_API_KEY");
        env::remove_var("LLM_GUARD_ENDPOINT");
        env::remove_var("LLM_GUARD_MODEL");
        env::remove_var("LLM_GUARD_DEPLOYMENT");
        env::remove_var("LLM_GUARD_PROJECT");
        env::remove_var("LLM_GUARD_WORKSPACE");
        env::remove_var("LLM_GUARD_TIMEOUT_SECS");
        env::remove_var("LLM_GUARD_MAX_RETRIES");
        env::remove_var("LLM_GUARD_API_VERSION");
    }

    #[test]
    fn prime_env_sets_missing_values() {
        let _guard = ENV_LOCK.lock().unwrap();
        reset_vars();

        let profile = ProviderProfile {
            name: "azure".into(),
            api_key: Some("azure-key".into()),
            endpoint: Some("https://example.azure.com".into()),
            model: Some("gpt-4o".into()),
            deployment: Some("security-deployment".into()),
            project: Some("proj".into()),
            workspace: Some("ws".into()),
            timeout_secs: Some(45),
            max_retries: Some(5),
            api_version: Some("2024-02-01".into()),
        };

        let mut entries = HashMap::new();
        entries.insert("azure".into(), profile);
        let profiles = ProviderProfiles { entries };

        profiles.prime_env("azure");

        assert_eq!(env::var("LLM_GUARD_PROVIDER").unwrap(), "azure");
        assert_eq!(env::var("LLM_GUARD_API_KEY").unwrap(), "azure-key");
        assert_eq!(
            env::var("LLM_GUARD_ENDPOINT").unwrap(),
            "https://example.azure.com"
        );
        assert_eq!(
            env::var("LLM_GUARD_DEPLOYMENT").unwrap(),
            "security-deployment"
        );
        assert_eq!(env::var("LLM_GUARD_TIMEOUT_SECS").unwrap(), "45");
        assert_eq!(env::var("LLM_GUARD_MAX_RETRIES").unwrap(), "5");
        assert_eq!(env::var("LLM_GUARD_API_VERSION").unwrap(), "2024-02-01");
        reset_vars();
    }

    #[test]
    fn apply_defaults_populates_missing_fields() {
        let _guard = ENV_LOCK.lock().unwrap();
        reset_vars();

        let profile = ProviderProfile {
            name: "azure".into(),
            api_key: Some("azure-key".into()),
            endpoint: Some("https://example.azure.com".into()),
            model: Some("gpt-4o".into()),
            deployment: Some("security-deployment".into()),
            project: Some("proj".into()),
            workspace: Some("ws".into()),
            timeout_secs: Some(60),
            max_retries: Some(4),
            api_version: Some("2024-02-01".into()),
        };

        let mut entries = HashMap::new();
        entries.insert("azure".into(), profile.clone());
        let profiles = ProviderProfiles { entries };

        let mut settings = LlmSettings {
            provider: "azure".into(),
            api_key: "azure-key".into(),
            endpoint: profile.endpoint.clone(),
            model: None,
            deployment: None,
            project: None,
            workspace: None,
            timeout_secs: None,
            max_retries: 2,
            api_version: None,
        };

        profiles.apply_defaults("azure", &mut settings);

        assert_eq!(settings.model.as_deref(), Some("gpt-4o"));
        assert_eq!(settings.deployment.as_deref(), Some("security-deployment"));
        assert_eq!(settings.project.as_deref(), Some("proj"));
        assert_eq!(settings.workspace.as_deref(), Some("ws"));
        assert_eq!(settings.timeout_secs, Some(60));
        assert_eq!(settings.max_retries, 4);
        assert_eq!(settings.api_version.as_deref(), Some("2024-02-01"));
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    match run().await {
        Ok(code) => process::exit(code),
        Err(err) => {
            eprintln!("Error: {:#}", err);
            process::exit(1);
        }
    }
}

async fn run() -> Result<i32> {
    init_tracing();
    let cli = Cli::parse();
    if cli.debug {
        env::set_var("LLM_GUARD_DEBUG", "1");
    } else {
        env::remove_var("LLM_GUARD_DEBUG");
    }
    let provider_profiles = ProviderProfiles::load(&cli.providers_config)?;
    match cli.command.unwrap_or(Commands::ListRules { json: false }) {
        Commands::ListRules { json } => {
            list_rules(&cli.rules_dir, json).await?;
            Ok(0)
        }
        Commands::Scan {
            file,
            json,
            tail,
            with_llm,
            provider,
            model,
            endpoint,
            deployment,
            project,
            workspace,
        } => {
            apply_config_overrides(cli.config_file.as_ref())?;
            scan_input(
                &cli.rules_dir,
                file.as_deref(),
                json,
                tail,
                with_llm,
                provider.as_deref(),
                model.as_deref(),
                endpoint.as_deref(),
                deployment.as_deref(),
                project.as_deref(),
                workspace.as_deref(),
                &provider_profiles,
            )
            .await
        }
        Commands::Health { provider, dry_run } => {
            run_health(&provider_profiles, provider.as_deref(), !dry_run).await
        }
    }
}

fn apply_config_overrides(config_path: Option<&PathBuf>) -> Result<()> {
    let Some(path) = config_path else {
        return Ok(());
    };
    let settings = Config::builder()
        .add_source(config::File::from(path.as_path()))
        .build()
        .context("failed to load configuration file")?;

    maybe_set_env(
        "LLM_GUARD_PROVIDER",
        settings.get_string("llm.provider").ok(),
    );
    maybe_set_env("LLM_GUARD_API_KEY", settings.get_string("llm.api_key").ok());
    maybe_set_env(
        "LLM_GUARD_ENDPOINT",
        settings.get_string("llm.endpoint").ok(),
    );
    maybe_set_env("LLM_GUARD_MODEL", settings.get_string("llm.model").ok());
    maybe_set_env(
        "LLM_GUARD_TIMEOUT_SECS",
        settings.get_string("llm.timeout_secs").ok(),
    );
    maybe_set_env(
        "LLM_GUARD_MAX_RETRIES",
        settings.get_string("llm.max_retries").ok(),
    );
    maybe_set_env(
        "LLM_GUARD_API_VERSION",
        settings.get_string("llm.api_version").ok(),
    );
    maybe_set_env(
        "LLM_GUARD_DEPLOYMENT",
        settings.get_string("llm.deployment").ok(),
    );
    maybe_set_env("LLM_GUARD_PROJECT", settings.get_string("llm.project").ok());
    maybe_set_env(
        "LLM_GUARD_WORKSPACE",
        settings.get_string("llm.workspace").ok(),
    );

    Ok(())
}

fn maybe_set_env(var: &str, value: Option<String>) {
    if std::env::var(var).is_ok() {
        return;
    }
    if let Some(value) = value {
        std::env::set_var(var, value);
    }
}

async fn list_rules(rules_dir: &Path, json: bool) -> Result<()> {
    let repo = FileRuleRepository::new(rules_dir);
    let mut rules = RuleRepository::load_rules(&repo)
        .await
        .with_context(|| format!("failed to load rules from {}", rules_dir.display()))?;
    rules.sort_by(|a, b| a.id.cmp(&b.id));
    if json {
        println!("{}", serde_json::to_string_pretty(&rules)?);
        return Ok(());
    }

    println!(
        "{} rule(s) loaded from {}",
        rules.len(),
        rules_dir.display()
    );
    for rule in rules {
        let kind = match rule.kind {
            RuleKind::Keyword => "keyword",
            RuleKind::Regex => "regex",
        };
        let window = rule
            .window
            .map(|w| format!(", window {}", w))
            .unwrap_or_default();
        println!(
            "- {id:<20} [{kind:7}] weight {weight:>5.1} :: {desc}{window}",
            id = rule.id,
            kind = kind,
            weight = rule.weight,
            desc = rule.description,
            window = window
        );
    }
    Ok(())
}

async fn scan_input(
    rules_dir: &Path,
    file: Option<&Path>,
    json: bool,
    tail: bool,
    with_llm: bool,
    provider_override: Option<&str>,
    model_override: Option<&str>,
    endpoint_override: Option<&str>,
    deployment_override: Option<&str>,
    project_override: Option<&str>,
    workspace_override: Option<&str>,
    provider_profiles: &ProviderProfiles,
) -> Result<i32> {
    let repo = Arc::new(FileRuleRepository::new(rules_dir));
    let scanner = Arc::new(DefaultScanner::new(Arc::clone(&repo)));

    let llm_client: Option<Arc<dyn LlmClient>> = if with_llm {
        let provider_hint = provider_override
            .map(|s| s.to_string())
            .or_else(|| std::env::var("LLM_GUARD_PROVIDER").ok())
            .unwrap_or_else(|| "openai".to_string());
        provider_profiles.prime_env(&provider_hint);

        let mut settings = match LlmSettings::from_env() {
            Ok(s) => s,
            Err(err) => {
                if provider_override
                    .map(|p| p.eq_ignore_ascii_case("noop"))
                    .unwrap_or(false)
                {
                    LlmSettings {
                        provider: provider_override.unwrap().to_string(),
                        api_key: String::new(),
                        endpoint: endpoint_override.map(|s| s.to_string()),
                        model: model_override.map(|s| s.to_string()),
                        deployment: None,
                        project: None,
                        workspace: None,
                        timeout_secs: Some(30),
                        max_retries: 2,
                        api_version: None,
                    }
                } else {
                    return Err(err);
                }
            }
        };
        if let Some(provider) = provider_override {
            settings.provider = provider.to_string();
        }
        let provider_for_defaults = settings.provider.clone();
        provider_profiles.apply_defaults(&provider_for_defaults, &mut settings);
        if let Some(model) = model_override {
            settings.model = Some(model.to_string());
        }
        if let Some(endpoint) = endpoint_override {
            settings.endpoint = Some(endpoint.to_string());
        }
        if let Some(deployment) = deployment_override {
            settings.deployment = Some(deployment.to_string());
        }
        if settings.deployment.is_none() {
            settings.deployment = std::env::var("LLM_GUARD_DEPLOYMENT").ok();
        }
        if let Some(project) = project_override {
            settings.project = Some(project.to_string());
        }
        if settings.project.is_none() {
            settings.project = std::env::var("LLM_GUARD_PROJECT").ok();
        }
        if let Some(workspace) = workspace_override {
            settings.workspace = Some(workspace.to_string());
        }
        if settings.workspace.is_none() {
            settings.workspace = std::env::var("LLM_GUARD_WORKSPACE").ok();
        }
        if let Ok(api_version) = std::env::var("LLM_GUARD_API_VERSION") {
            settings.api_version = Some(api_version);
        }
        let client = build_client(&settings)?;
        Some(client.into())
    } else {
        None
    };

    if tail {
        let file = file.ok_or_else(|| anyhow!("--tail requires --file to specify a path"))?;
        tail_file(scanner, file, json, llm_client).await
    } else {
        let text = read_input(file)
            .await
            .with_context(|| "failed to read input for scanning")?;
        let mut report = scanner.scan(&text).await?;
        if let Some(client) = llm_client.as_ref() {
            let verdict = client.enrich(&text, &report).await?;
            report.llm_verdict = Some(verdict);
        }
        let rendered = render_report(
            &report,
            if json {
                OutputFormat::Json
            } else {
                OutputFormat::Human
            },
        )?;
        println!("{}", rendered);
        Ok(exit_code_for_band(report.risk_band))
    }
}

async fn read_input(path: Option<&Path>) -> Result<String> {
    if let Some(path) = path {
        Ok(fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read input file {}", path.display()))?)
    } else {
        let mut buf = String::new();
        let mut stdin = io::stdin();
        stdin
            .read_to_string(&mut buf)
            .await
            .context("failed to read from stdin")?;
        Ok(buf)
    }
}

async fn tail_file(
    scanner: Arc<DefaultScanner<FileRuleRepository>>,
    path: &Path,
    json: bool,
    llm_client: Option<Arc<dyn LlmClient>>,
) -> Result<i32> {
    let mut last_snapshot = String::new();
    let mut last_code = 0;
    loop {
        let contents = fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read tailed file {}", path.display()))?;
        if contents != last_snapshot {
            last_snapshot = contents.clone();
            let mut report = scanner.scan(&contents).await?;
            if let Some(client) = llm_client.as_ref() {
                let verdict = client.enrich(&contents, &report).await?;
                report.llm_verdict = Some(verdict);
            }
            let rendered = render_report(
                &report,
                if json {
                    OutputFormat::Json
                } else {
                    OutputFormat::Human
                },
            )?;
            println!("\n=== {} ===\n{}", path.display(), rendered);
            last_code = exit_code_for_band(report.risk_band);
        }

        tokio::select! {
            _ = sleep(Duration::from_secs(2)) => {},
            _ = signal::ctrl_c() => {
                eprintln!("Stopping tail for {}", path.display());
                return Ok(last_code);
            }
        }
    }
}

fn exit_code_for_band(band: RiskBand) -> i32 {
    match band {
        RiskBand::Low => 0,
        RiskBand::Medium => 2,
        RiskBand::High => 3,
    }
}

async fn run_health(
    profiles: &ProviderProfiles,
    provider_filter: Option<&str>,
    perform_call: bool,
) -> Result<i32> {
    let mut targets = if let Some(filter) = provider_filter {
        if let Some(profile) = profiles.get(filter) {
            vec![profile.name.clone()]
        } else {
            vec![filter.to_string()]
        }
    } else if !profiles.is_empty() {
        profiles.names()
    } else if let Ok(env_provider) = env::var("LLM_GUARD_PROVIDER") {
        vec![env_provider]
    } else {
        bail!("no providers configured; supply --provider or create llm_providers.yaml");
    };

    targets.sort();
    targets.dedup();

    let mut failed = false;
    for provider in targets {
        println!("Checking provider {provider}...");
        match check_provider(profiles, &provider, perform_call).await {
            Ok(()) => println!("  ok"),
            Err(err) => {
                failed = true;
                eprintln!("  failed: {err:#}");
            }
        }
    }

    Ok(if failed { 1 } else { 0 })
}

async fn check_provider(
    profiles: &ProviderProfiles,
    provider: &str,
    perform_call: bool,
) -> Result<()> {
    let profile_snapshot = profiles.get(provider).cloned();
    let canonical_provider = profile_snapshot
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| provider.to_string());

    let mut guard = EnvGuard::new();
    guard.set("LLM_GUARD_PROVIDER", &canonical_provider);
    if let Some(profile) = profile_snapshot.as_ref() {
        guard.maybe_set("LLM_GUARD_API_KEY", profile.api_key.as_deref());
        guard.maybe_set("LLM_GUARD_ENDPOINT", profile.endpoint.as_deref());
        guard.maybe_set("LLM_GUARD_MODEL", profile.model.as_deref());
        guard.maybe_set("LLM_GUARD_DEPLOYMENT", profile.deployment.as_deref());
        guard.maybe_set("LLM_GUARD_PROJECT", profile.project.as_deref());
        guard.maybe_set("LLM_GUARD_WORKSPACE", profile.workspace.as_deref());
        if let Some(timeout) = profile.timeout_secs {
            guard.set("LLM_GUARD_TIMEOUT_SECS", &timeout.to_string());
        }
        if let Some(retries) = profile.max_retries {
            guard.set("LLM_GUARD_MAX_RETRIES", &retries.to_string());
        }
        guard.maybe_set("LLM_GUARD_API_VERSION", profile.api_version.as_deref());
    }

    let mut settings = LlmSettings::from_env()?;
    let provider_for_defaults = settings.provider.clone();
    profiles.apply_defaults(&provider_for_defaults, &mut settings);
    drop(guard);

    let client = build_client(&settings)?;
    if perform_call {
        let report = dummy_report();
        let _ = client
            .enrich("Health check probe", &report)
            .await
            .context("LLM enrich call failed")?;
    }

    Ok(())
}

fn dummy_report() -> ScanReport {
    ScanReport::from_breakdown(
        Vec::new(),
        0,
        None,
        ScoreBreakdown::default(),
        &RiskThresholds::default(),
    )
}

fn init_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,tokio=warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .try_init();
}
