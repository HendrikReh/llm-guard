use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use llm_guard_core::{
    build_client, render_report, DefaultScanner, FileRuleRepository, LlmClient, LlmSettings,
    OutputFormat, RiskBand, RuleKind, RuleRepository, Scanner,
};
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
    },
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
            )
            .await
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
) -> Result<i32> {
    let repo = Arc::new(FileRuleRepository::new(rules_dir));
    let scanner = Arc::new(DefaultScanner::new(Arc::clone(&repo)));

    let llm_client: Option<Arc<dyn LlmClient>> = if with_llm {
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
                        timeout_secs: Some(30),
                        max_retries: 2,
                    }
                } else {
                    return Err(err);
                }
            }
        };
        if let Some(provider) = provider_override {
            settings.provider = provider.to_string();
        }
        if let Some(model) = model_override {
            settings.model = Some(model.to_string());
        }
        if let Some(endpoint) = endpoint_override {
            settings.endpoint = Some(endpoint.to_string());
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

fn init_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,tokio=warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .try_init();
}
