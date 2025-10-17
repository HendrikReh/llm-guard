use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use std::fs::write;

fn set_common_env(provider: &str) {
    env::set_var("LLM_GUARD_PROVIDER", provider);
    env::set_var("LLM_GUARD_API_KEY", "test-key");
}

#[test]
fn scan_with_llm_noop_provider() {
    set_common_env("noop");
    let mut cmd = Command::cargo_bin("llm-guard-cli").unwrap();
    cmd.args(["scan", "--with-llm"])
        .write_stdin("hello world")
        .assert()
        .success()
        .stdout(predicate::str::contains("Risk Score"))
        .stdout(predicate::str::contains("LLM Verdict"));
}

#[test]
fn scan_with_config_file() {
    let file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    write(
        file.path(),
        "llm = { provider = \"noop\", model = \"config-model\" }",
    )
    .unwrap();

    env::remove_var("LLM_GUARD_PROVIDER");
    env::remove_var("LLM_GUARD_API_KEY");

    let mut cmd = Command::cargo_bin("llm-guard-cli").unwrap();
    cmd.args([
        "--config",
        file.path().to_str().unwrap(),
        "scan",
        "--with-llm",
    ])
    .write_stdin("test input")
    .assert()
    .success()
    .stdout(predicate::str::contains("LLM Verdict"));
}
