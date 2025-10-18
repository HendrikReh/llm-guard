use assert_cmd::Command;
use once_cell::sync::Lazy;
use predicates::str::contains;
use std::env;
use std::fs::write;
use std::sync::Mutex;

static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn reset_env() {
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
    env::remove_var("LLM_GUARD_DEBUG");
}

#[test]
fn health_check_with_noop_profile() {
    let _guard = ENV_LOCK.lock().unwrap();
    reset_env();

    let file = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();

    write(file.path(), "providers:\n  - name: \"noop\"\n").unwrap();

    let mut cmd = Command::cargo_bin("llm-guard-cli").unwrap();
    cmd.args([
        "--providers-config",
        file.path().to_str().unwrap(),
        "health",
    ])
    .assert()
    .success()
    .stdout(contains("Checking provider noop"))
    .stdout(contains("ok"));
}

#[test]
#[cfg_attr(
    target_os = "macos",
    ignore = "SystemConfiguration dynamic store unavailable in sandboxed subprocess"
)]
fn health_check_with_openai_profile_dry_run() {
    let _guard = ENV_LOCK.lock().unwrap();
    reset_env();

    let file = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();

    write(
        file.path(),
        "providers:\n  - name: \"openai\"\n    api_key: \"test-key\"\n",
    )
    .unwrap();

    env::set_var("LLM_GUARD_API_KEY", "test-key");

    let mut cmd = Command::cargo_bin("llm-guard-cli").unwrap();
    cmd.args([
        "--providers-config",
        file.path().to_str().unwrap(),
        "health",
        "--provider",
        "openai",
        "--dry-run",
    ])
    .assert()
    .success()
    .stdout(contains("Checking provider openai"))
    .stdout(contains("ok"));

    reset_env();
}
