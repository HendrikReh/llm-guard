use assert_cmd::Command;
use predicates::prelude::*;
use std::env;

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
