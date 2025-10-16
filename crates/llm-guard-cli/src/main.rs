use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use llm_guard_core::{FileRuleRepository, RuleKind, RuleRepository};
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
    /// List all loaded rules
    ListRules {
        /// Emit rules as JSON instead of human-readable text
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::ListRules { json: false }) {
        Commands::ListRules { json } => list_rules(&cli.rules_dir, json).await?,
    }
    Ok(())
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

fn init_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,tokio=warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .try_init();
}
