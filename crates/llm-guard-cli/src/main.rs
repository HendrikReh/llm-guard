use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use llm_guard_core::{
    render_report, DefaultScanner, FileRuleRepository, OutputFormat, RiskBand, RuleKind,
    RuleRepository, Scanner,
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
        } => scan_input(&cli.rules_dir, file.as_deref(), json, tail, with_llm).await,
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
) -> Result<i32> {
    if with_llm {
        bail!("--with-llm is not implemented yet; stay tuned for Phase 6.");
    }
    let repo = Arc::new(FileRuleRepository::new(rules_dir));
    let scanner = Arc::new(DefaultScanner::new(Arc::clone(&repo)));

    if tail {
        let file = file.ok_or_else(|| anyhow!("--tail requires --file to specify a path"))?;
        tail_file(scanner, file, json).await
    } else {
        let text = read_input(file)
            .await
            .with_context(|| "failed to read input for scanning")?;
        let report = scanner.scan(&text).await?;
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
) -> Result<i32> {
    let mut last_snapshot = String::new();
    let mut last_code = 0;
    loop {
        let contents = fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read tailed file {}", path.display()))?;
        if contents != last_snapshot {
            last_snapshot = contents.clone();
            let report = scanner.scan(&contents).await?;
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
