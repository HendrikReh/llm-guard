use std::{fs, path::PathBuf, sync::Arc};

use insta::assert_json_snapshot;
use llm_guard_core::scanner::{
    default_scanner::DefaultScanner, file_repository::FileRuleRepository, Scanner,
};
use serde_json::json;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rules_dir() -> PathBuf {
    workspace_root().join("rules")
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

async fn scan_fixture(name: &str) -> serde_json::Value {
    let fixture_path = fixture_dir().join(name);
    let input = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", fixture_path.display()));

    let repo = Arc::new(FileRuleRepository::new(rules_dir()));
    let scanner = DefaultScanner::new(Arc::clone(&repo));

    let report = scanner
        .scan(&input)
        .await
        .unwrap_or_else(|err| panic!("scan failed for fixture {}: {err:#}", name));

    json!({
        "fixture": name,
        "risk_score": report.risk_score,
        "risk_band": report.risk_band,
        "normalized_len": report.normalized_len,
        "findings": report.findings.iter().map(|finding| json!({
            "rule_id": finding.rule_id,
            "weight": finding.weight,
            "excerpt": finding.excerpt,
        })).collect::<Vec<_>>(),
        "score_breakdown": {
            "raw_total": report.score_breakdown.raw_total,
            "adjusted_total": report.score_breakdown.adjusted_total,
            "length_factor": report.score_breakdown.length_factor,
            "families": report.score_breakdown.family_contributions,
        }
    })
}

#[tokio::test(flavor = "current_thread")]
async fn safe_prompt_snapshot() {
    let snapshot = scan_fixture("safe_prompt.txt").await;
    assert_json_snapshot!("safe_prompt", snapshot);
}

#[tokio::test(flavor = "current_thread")]
async fn suspicious_prompt_snapshot() {
    let snapshot = scan_fixture("suspicious_prompt.txt").await;
    assert_json_snapshot!("suspicious_prompt", snapshot);
}

#[tokio::test(flavor = "current_thread")]
async fn malicious_prompt_snapshot() {
    let snapshot = scan_fixture("malicious_prompt.txt").await;
    assert_json_snapshot!("malicious_prompt", snapshot);
}
