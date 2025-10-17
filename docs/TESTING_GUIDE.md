# Testing Guide

> **AI-Assisted Development Note:** This testing strategy was developed collaboratively with GPT-5 Codex and Claude Code. See [AGENTS.md](../AGENTS.md) for testing conventions and quality expectations.

This document describes the testing strategy, test execution, and quality assurance practices for LLM-Guard.

## Test Suite Overview

LLM-Guard has comprehensive test coverage across multiple levels following the test pyramid strategy:

| Test Type | Count | Status | Purpose | Execution Time |
|-----------|-------|--------|---------|----------------|
| Unit Tests | 28 | ✅ Pass | Core logic validation | <100ms |
| Integration Tests | 2 | ✅ Pass | End-to-end CLI scenarios | ~400ms |
| Network Tests | 8 | 🔶 Ignored | Provider HTTP client testing | ~2s |
| TLS Tests | 2 | 🔶 Ignored (macOS) | Rig adapter builder tests | N/A |

**Total:** 40 tests (30 active, 10 ignored)
**Test Execution:** `cargo test` runs in <500ms
**Coverage:** ~85% (estimated)

## Quick Reference

### Most Common Commands

```bash
# Run all active tests (fast, no network)
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Run tests with tracing enabled
RUST_LOG=debug cargo test -- --nocapture

# Run tests in watch mode (requires cargo-watch)
cargo watch -x test
```

### Selective Test Execution

```bash
# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run tests in specific package
cargo test --package llm-guard-core
cargo test --package llm-guard-cli

# Run specific test by name
cargo test risk_score

# Run tests matching pattern
cargo test scanner::
```

### Ignored Tests

```bash
# Run ONLY ignored tests
cargo test -- --ignored

# Run ALL tests (normal + ignored)
cargo test -- --include-ignored

# Run specific ignored test
cargo test openai::tests::enrich_parses_successful_response -- --ignored

# Run all network tests
cargo test llm:: -- --ignored
```

### Alias Commands (if configured in .cargo/config.toml)

```bash
# Configured workspace aliases
cargo test-all          # Test all packages with all features
cargo cov               # Generate coverage report
```

## Test Categories

### Unit Tests (28 tests) ✅

**Location:** `crates/llm-guard-core/src/**/*.rs` (inline with implementation)

**What They Test:**
- **Scanner Engine:**
  - Pattern matching with Aho-Corasick and regex
  - Finding generation with proper span and excerpt extraction
  - Character boundary handling for Unicode
  - Zero-width match filtering
- **Risk Scoring ([ADR-0001](./ADR/0001-heuristic-risk-scoring.md)):**
  - Weight aggregation with diminishing returns
  - Length normalization (clamp to 0.5-1.5)
  - Score clamping to [0, 100] range
  - Risk band threshold validation
- **Rule Management:**
  - Rule loading from JSON and text files
  - Duplicate ID detection
  - Weight bound validation (0.0-20.0)
  - Regex compilation validation
- **Reporting:**
  - Human-readable formatting with colors
  - JSON serialization of ScanReport
  - Finding attribution and breakdown
- **LLM Clients:**
  - String truncation logic (≤800 chars)
  - Configuration parsing and validation
  - Provider selection and defaults

**Example Test:**
```rust
#[test]
fn scan_report_clamps_scores() {
    // Verify scores never exceed 100.0 even with high weights
    let mut report = ScanReport::new();
    report.risk_score = 150.0;
    assert!(report.risk_score <= 100.0);
    assert!(report.risk_score >= 0.0);
}
```

**Run:** `cargo test --lib`

### Integration Tests (2 tests) ✅

**Location:** `crates/llm-guard-cli/tests/`

**What They Test:**
- **End-to-End CLI Workflows:**
  - Complete scan pipeline from input → detection → scoring → output
  - CLI argument parsing and validation
  - Exit code behavior (0 safe, 2 medium, 3 high, 1 error)
- **Configuration Integration:**
  - Config file loading and precedence rules
  - Environment variable overrides
  - Feature flag handling (`--with-llm`, `--json`, `--tail`)
- **LLM Integration (noop provider):**
  - Verify `--with-llm` flag triggers LLM analysis path
  - Test verdict merging into ScanReport
  - No actual API calls (uses noop/mock adapter)

**Example Test Scenario:**
```bash
# Build CLI binary first
cargo build

# Run integration test
./target/debug/llm-guard-cli scan --file samples/test.txt --with-llm

# Expected: Clean exit, report generated, noop verdict included
```

**Run:** `cargo test --test '*'`

### Network Tests (8 tests - Ignored) 🔶

**Why Ignored:** Require loopback networking for mock HTTP servers (not available in sandboxed environments)

**Location:** `crates/llm-guard-core/src/llm/**/tests`

**What They Test:**
- **HTTP Client Behavior ([ADR-0003](./ADR/0003-optional-llm-integration.md)):**
  - Request formation (headers, auth, JSON payload)
  - Response parsing and deserialization
  - Error handling for 4xx/5xx status codes
  - Retry logic with exponential backoff
  - Timeout enforcement
- **Provider-Specific Integration:**
  - **OpenAI:** GPT-4 completion parsing (2 tests)
  - **Anthropic:** Claude message format handling (2 tests)
  - **Azure OpenAI:** Deployment-specific endpoints (2 tests)
  - **Gemini:** GenerateContent API integration (2 tests)

**Running Network Tests:**
```bash
# Run all network tests (requires loopback access)
cargo test -- --ignored

# Run specific provider tests
cargo test openai::tests -- --ignored
cargo test anthropic::tests -- --ignored
cargo test azure::tests -- --ignored
cargo test gemini::tests -- --ignored
```

**Test Infrastructure:**
- Uses `wiremock` for HTTP mocking
- Mock servers bind to `127.0.0.1:0` (random port)
- Tests are async (`#[tokio::test]`)
- Marked with `#[ignore = "requires loopback networking"]`

**When to Run:**
- Before releasing new LLM provider support
- After changing HTTP client configuration
- When debugging API integration issues
- **Not required** for normal development (unit tests cover logic)

### TLS Tests (2 tests - Ignored on macOS) 🔶

**Why Ignored:** `reqwest` default TLS stack (native-tls) unavailable in macOS sandbox due to Security framework restrictions

**Location:** `crates/llm-guard-core/src/llm/rig_adapter.rs`

**What They Test:**
- **Rig Adapter Builder ([ADR-0003](./ADR/0003-optional-llm-integration.md)):**
  - OpenAI client initialization via `rig-core`
  - Default model selection behavior
  - Explicit model override configuration
  - TLS certificate validation during HTTPS setup

**Running TLS Tests:**
```bash
# On Linux (should work)
cargo test rig_adapter::tests -- --ignored

# On macOS (fails in sandbox due to native-tls Security framework)
# ❌ Error: "TLS backend cannot be initialized"
```

**macOS Sandbox Limitation:**
The default `reqwest` TLS backend (`native-tls`) requires access to macOS Security framework, which is restricted in sandboxed environments (including some terminal sessions and CI runners).

**Workaround for macOS:**
Switch to pure-Rust TLS implementation (`rustls`):

```toml
# In llm-guard-core/Cargo.toml
[dependencies]
reqwest = { version = "0.11", features = ["rustls-tls"], default-features = false }
rig-core = { version = "0.22", default-features = false, features = ["rustls-tls"] }
```

**When to Run:**
- On Linux CI/CD pipelines (works reliably)
- Before releasing changes to rig adapter integration
- **Not required** on macOS development machines (already marked ignored)

## Running Tests in CI/CD

### GitHub Actions Example

```yaml
- name: Run tests
  run: |
    # Run non-ignored tests (fast, no network)
    cargo test --workspace --all-features

    # Optionally run ignored tests if network available
    cargo test --workspace --all-features -- --ignored
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running tests..."
cargo test --quiet

if [ $? -ne 0 ]; then
    echo "Tests failed. Commit aborted."
    exit 1
fi
```

## Test Fixtures

Test fixtures provide realistic inputs for integration tests and manual testing, ensuring detection rules work as expected across different risk levels.

### Sample Prompts

**Location:** `crates/llm-guard-cli/tests/fixtures/` (if present) or `samples/`

**Fixture Categories:**
- **`safe.txt`** - Legitimate prompt with no policy violations
  - Expected: Risk score < 30 (Low)
  - Exit code: 0
- **`suspicious.txt`** - Prompt with override phrases or mild jailbreak attempts
  - Expected: Risk score 30-70 (Medium)
  - Exit code: 2
  - Examples: "Ignore previous instructions", "Pretend you are"
- **`malicious.txt`** - Prompt with system prompt leaks + obfuscation
  - Expected: Risk score > 70 (High)
  - Exit code: 3
  - Examples: Combined DAN + encoding + exfiltration patterns

**Usage:**
```bash
# Test against fixtures
cargo run -- scan --file samples/safe.txt
cargo run -- scan --file samples/suspicious.txt --json
cargo run -- scan --file samples/malicious.txt --with-llm
```

### Rule Packs

**Location:** `rules/`

**Current Rule Files:**
- **`keywords.txt`** - Simple keyword patterns for fast Aho-Corasick matching
  - Format: One keyword per line
  - Example: `DAN mode`, `ignore previous`, `system prompt`
- **`patterns.json`** - Complex regex rules with metadata
  - Format: JSON array of rule objects
  - Fields: `id`, `pattern`, `weight`, `description`, `family`

**Extending Rule Packs:**
See [PRD.md](../PRD.md) Section 6.2 for rule authoring guidelines and weight tuning recommendations.

## Writing New Tests

When adding new features or fixing bugs, follow these patterns to maintain test coverage and consistency.

### Unit Test Template

**Location:** Inline with implementation in `src/**/*.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_describes_behavior() {
        // Arrange: Set up test data and preconditions
        let input = "test input";
        let expected = "expected output";

        // Act: Execute the function under test
        let result = function_under_test(input);

        // Assert: Verify the behavior
        assert_eq!(result, expected);
    }

    #[test]
    fn test_error_handling() {
        let invalid_input = "";

        let result = function_under_test(invalid_input);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Input cannot be empty"
        );
    }
}
```

**Best Practices:**
- Test function name should describe the behavior being tested
- Use Arrange-Act-Assert pattern
- Test both success and error paths
- Add edge cases (empty strings, Unicode, boundary values)

### Integration Test Template

**Location:** `crates/llm-guard-cli/tests/*.rs`

```rust
// tests/integration_test.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_scan_with_output() {
    let mut cmd = Command::cargo_bin("llm-guard-cli").unwrap();

    cmd.arg("scan")
       .arg("--file")
       .arg("tests/fixtures/test.txt")
       .arg("--json");

    cmd.assert()
       .success()
       .stdout(predicate::str::contains("risk_score"));
}

#[test]
fn test_cli_exit_code_high_risk() {
    let mut cmd = Command::cargo_bin("llm-guard-cli").unwrap();

    cmd.arg("scan")
       .arg("--file")
       .arg("tests/fixtures/malicious.txt");

    cmd.assert()
       .code(3); // High risk exit code
}
```

**Best Practices:**
- Test CLI argument combinations
- Verify exit codes match specification
- Test both human-readable and JSON output
- Use `predicates` for flexible output matching

### Network Test Template

**Location:** `crates/llm-guard-core/src/llm/**/tests`

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};

#[tokio::test]
#[ignore = "requires loopback networking"]
async fn test_provider_success() {
    // Arrange: Start mock HTTP server
    let server = MockServer::start().await;

    // Configure mock response
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({
                    "choices": [{"message": {"content": "benign"}}]
                }))
        )
        .mount(&server)
        .await;

    // Act: Test client against mock server
    let client = ProviderClient::new(&server.uri(), "test-key");
    let result = client.enrich_findings("test prompt").await;

    // Assert: Verify successful parsing
    assert!(result.is_ok());
    assert_eq!(result.unwrap().verdict, "benign");
}

#[tokio::test]
#[ignore = "requires loopback networking"]
async fn test_provider_retry_on_500() {
    let server = MockServer::start().await;

    // First request fails, second succeeds
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&server)
        .await;

    let client = ProviderClient::new(&server.uri(), "test-key");
    let result = client.enrich_findings("test").await;

    // Should succeed after retry
    assert!(result.is_ok());
}
```

**Best Practices:**
- Always mark with `#[ignore]` and descriptive reason
- Use `wiremock` for HTTP mocking (already in dependencies)
- Test both success and failure scenarios
- Test retry logic and timeout behavior
- Use `#[tokio::test]` for async tests

## Test Coverage

Test coverage helps identify untested code paths and ensure comprehensive validation of critical logic.

### Generating Coverage Reports

**Install Coverage Tool:**
```bash
cargo install cargo-llvm-cov
```

**Generate Reports:**
```bash
# HTML coverage report (interactive, detailed)
cargo llvm-cov --workspace --html

# Open report in browser
open target/llvm-cov/html/index.html

# Terminal summary (quick overview)
cargo llvm-cov --workspace

# LCOV format for CI integration (Codecov, Coveralls)
cargo llvm-cov --workspace --lcov --output-path lcov.info

# JSON format for programmatic parsing
cargo llvm-cov --workspace --json --output-path coverage.json
```

**Include Ignored Tests (optional):**
```bash
# Generate coverage including network/TLS tests
# (Only works if loopback networking available)
cargo llvm-cov --workspace --html -- --include-ignored
```

### Coverage Targets

**Current Coverage:** ~85% (estimated based on test suite)

**Component-Level Goals:**
- **Core scanner logic:** >90% (Pattern matching, finding generation)
- **Risk scoring:** >95% (Heuristic algorithm is deterministic)
- **Rule management:** >85% (Parsing, validation, caching)
- **CLI argument parsing:** >80% (Multiple flag combinations)
- **LLM adapters:** >70% (Network tests ignored, harder to cover)
- **Reporters:** >80% (Human-readable and JSON formatting)

**Priority for 100% Coverage:**
- Error handling paths (especially in scanner and rule loader)
- Boundary conditions (empty inputs, Unicode edge cases)
- Risk score clamping and normalization logic

**Acceptable Lower Coverage:**
- Network adapter code (tested manually, ignored in CI)
- CLI main.rs boilerplate (integration tests cover this)
- Generated code (serde derives, etc.)

## Debugging Failed Tests

When tests fail, use these techniques to diagnose and fix issues quickly.

### Show Test Output

By default, `cargo test` hides stdout/stderr from passing tests. To see all output:

```bash
# Show println! and dbg! output
cargo test -- --nocapture

# Enable tracing/logging output
RUST_LOG=debug cargo test -- --nocapture

# Show output for specific test
cargo test test_name -- --nocapture

# Show output with timestamps (useful for async tests)
RUST_LOG=debug,tokio=trace cargo test -- --nocapture
```

### Run Single Test

**By Exact Name:**
```bash
# Find test name from failure output, then run isolated
cargo test test_scanner_finds_keywords -- --exact --nocapture
```

**By Pattern:**
```bash
# Run all tests in a module
cargo test scanner::tests::

# Run all tests containing "risk"
cargo test risk

# Run all tests in specific file (integration tests)
cargo test --test cli_integration
```

### Common Test Failures

**1. "Address already in use" (Network tests):**
```bash
# Run tests serially to avoid port conflicts
cargo test -- --test-threads=1 --ignored
```

**2. "Assertion failed" with unclear diff:**
```bash
# Use pretty_assertions for better error messages
# Already in dev-dependencies, just use assert_eq! as normal
```

**3. Async test hangs:**
```bash
# Add timeout to test
#[tokio::test(flavor = "multi_thread")]
#[timeout(Duration::from_secs(5))]
async fn test_with_timeout() { ... }
```

**4. Flaky test (intermittent failures):**
```bash
# Run test multiple times to reproduce
for i in {1..50}; do
  cargo test test_name || break
done
```

### Update Snapshot Tests (if using insta)

If project adds `insta` for snapshot testing in the future:

```bash
# Review and accept snapshot changes
cargo insta review

# Accept all changes (use with caution)
cargo insta accept

# Reject all pending snapshots
cargo insta reject
```

## Performance Testing

Performance testing ensures the scanner remains fast even with large inputs and complex rule sets.

### Benchmarks

**Location:** `crates/llm-guard-core/benches/` (if present)

**Setup:**
```bash
# Install Criterion (if not in dependencies)
cargo install cargo-criterion
```

**Run Benchmarks:**
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench scanner_bench

# Run with baseline comparison
cargo bench --bench scanner_bench -- --baseline main

# Generate HTML report
cargo bench -- --noplot
```

**Benchmark Targets:**
- Scanner performance with 100/1000/10000 keywords
- Regex compilation and matching overhead
- Risk score calculation with various finding counts
- Rule loading and caching

**Expected Performance:**
- Scanner: <10ms for 10K characters, 100 rules
- Risk scoring: <1ms for typical finding sets
- Rule loading: <50ms for 1000 rules (cached afterward)

### Load Testing

**Manual Load Testing:**
```bash
# Build release binary
cargo build --release

# Generate large test file
python3 -c "print('test prompt ' * 10000)" > large.txt

# Measure scan time
time ./target/release/llm-guard-cli scan --file large.txt

# Expected: <100ms for 10K characters, <500ms for 100K characters
```

**Stress Testing with Multiple Runs:**
```bash
# Run 100 scans and measure average time
for i in {1..100}; do
  /usr/bin/time -p ./target/release/llm-guard-cli scan --file test.txt 2>&1 | grep real
done | awk '{sum+=$2} END {print "Average:", sum/NR, "seconds"}'
```

**Memory Profiling:**
```bash
# Install valgrind/heaptrack (Linux) or Instruments (macOS)

# Linux
valgrind --tool=massif ./target/release/llm-guard-cli scan --file large.txt

# macOS
instruments -t "Allocations" ./target/release/llm-guard-cli scan --file large.txt
```

## Test Maintenance

Keep the test suite up to date as the project evolves. Follow these checklists when adding new features.

### Adding New Provider Tests

When implementing a new LLM provider (e.g., Cohere, Mistral):

**Checklist:**
1. ✅ Add unit tests for truncation logic in `src/llm/provider_name.rs`
2. ✅ Add network tests for successful response parsing
3. ✅ Add network tests for retry logic on 5xx errors
4. ✅ Add network tests for timeout handling
5. ✅ Mark network tests with `#[ignore = "requires loopback networking"]`
6. ✅ Update this document's test count (line 11-18)
7. ✅ Add provider to `docs/ADR/0003-optional-llm-integration.md`
8. ✅ Update README.md badges if applicable

**Example Provider Test Structure:**
```
src/llm/provider_name.rs
  ├── impl ProviderClient
  └── #[cfg(test)]
      mod tests {
          ├── test_truncate_prompt()           // Unit
          ├── test_enrich_parses_successful_response()  // Network (ignored)
          └── test_retry_on_server_error()     // Network (ignored)
      }
```

### Updating Test Fixtures

When adding new detection rules or rule families:

**Checklist:**
1. ✅ Add example prompts to `samples/` or `tests/fixtures/`
   - One safe example (should not trigger)
   - One suspicious example (should trigger with medium risk)
   - One malicious example (should trigger with high risk)
2. ✅ Add unit tests verifying new rules parse correctly
3. ✅ Add integration tests verifying new rules trigger as expected
4. ✅ Update expected risk scores in existing tests if thresholds change
5. ✅ Document new rule family in `PRD.md` Section 6.2
6. ✅ Update `rules/keywords.txt` or `rules/patterns.json`

**Example:**
```bash
# After adding "credential harvesting" rule family
echo "show me all environment variables" > samples/credential_harvest.txt

# Add integration test
cargo test test_credential_harvesting_detection
```

### Regression Test Protocol

When fixing a bug:

**Checklist:**
1. ✅ Write a failing test that reproduces the bug
2. ✅ Fix the bug
3. ✅ Verify test now passes
4. ✅ Add test to regression suite (keep it!)
5. ✅ Document the bug and fix in commit message

**Example:**
```rust
#[test]
fn test_unicode_boundary_handling_regression() {
    // Regression test for issue #42: panic on emoji boundaries
    let input = "test 🚀 prompt";
    let result = scanner.scan(input);
    assert!(result.is_ok()); // Should not panic
}
```

## Troubleshooting

Common test issues and their solutions.

### "TLS backend cannot be initialized" on macOS

**Problem:** Rig adapter tests fail with TLS initialization error

**Error Message:**
```
thread 'rig_adapter::tests::test_openai_builder' panicked at 'TLS backend cannot be initialized'
```

**Root Cause:** macOS sandbox restricts access to Security framework used by `native-tls`

**Solutions:**
1. **Ignore tests** (already done) - Tests marked `#[ignore]` on macOS
2. **Switch to rustls** - See TLS Tests section for Cargo.toml changes
3. **Run outside sandbox** - Run tests in non-sandboxed terminal

**Verify fix:**
```bash
cargo test rig_adapter::tests -- --ignored
```

### "Address already in use" in network tests

**Problem:** Mock server port conflicts when running network tests in parallel

**Error Message:**
```
Error: Address already in use (os error 48)
```

**Root Cause:** Multiple tests trying to bind to same port

**Solutions:**
1. **Use random ports** (recommended):
   ```rust
   let server = MockServer::start().await; // Binds to 127.0.0.1:0 (random port)
   ```

2. **Run serially**:
   ```bash
   cargo test -- --test-threads=1 --ignored
   ```

3. **Add test isolation**:
   ```rust
   #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
   ```

### Slow test execution

**Problem:** `cargo test` takes too long (>5 seconds)

**Expected:** <500ms for non-ignored tests

**Diagnosis:**
```bash
# Show which tests are slow
cargo test -- --nocapture --test-threads=1 | grep "test result:"
```

**Solutions:**

1. **Use nextest** (faster test runner):
   ```bash
   cargo install cargo-nextest
   cargo nextest run
   # 30-50% faster than cargo test
   ```

2. **Increase parallelism**:
   ```bash
   # Default is CPU count, increase if I/O bound
   cargo test -- --test-threads=8
   ```

3. **Profile tests**:
   ```bash
   cargo test -- -Z unstable-options --report-time
   ```

4. **Skip slow tests during development**:
   ```bash
   # Mark slow tests
   #[test]
   #[ignore = "slow"]
   fn test_large_corpus_scan() { ... }

   # Run only fast tests
   cargo test --lib
   ```

### Tests pass locally but fail in CI

**Problem:** Tests succeed on your machine but fail on GitHub Actions

**Common Causes:**

1. **Network tests not ignored:**
   ```bash
   # Check CI logs for "requires loopback networking"
   # Solution: Ensure all network tests marked #[ignore]
   ```

2. **Platform-specific behavior:**
   ```bash
   # Use conditional compilation
   #[cfg(target_os = "macos")]
   #[ignore = "macOS-specific TLS issue"]
   ```

3. **Environment variables missing:**
   ```bash
   # CI may not have .env file
   # Solution: Set env vars in GitHub Actions workflow
   ```

4. **Timing/race conditions:**
   ```bash
   # Add explicit waits in async tests
   tokio::time::sleep(Duration::from_millis(100)).await;
   ```

### "No rule ID" errors

**Problem:** Rule loading fails with missing ID field

**Error Message:**
```
Error: Rule at index 5 is missing required 'id' field
```

**Solution:** Validate `rules/*.json` files:
```bash
# Check JSON syntax
jq empty rules/patterns.json

# Verify all rules have IDs
jq '.[] | select(.id == null)' rules/patterns.json
```

## Related Documentation

- [ADR-0001](./ADR/0001-heuristic-risk-scoring.md) - Risk scoring algorithm (tests verify this)
- [ADR-0003](./ADR/0003-optional-llm-integration.md) - LLM integration (network tests)
- [ADR-0004](./ADR/0004-aho-corasick-regex-detection.md) - Pattern matching (scanner tests)
- [`PLAN.md`](../PLAN.md) Phase 7 - Quality Engineering phase

## Contributing Tests

When contributing code:

1. ✅ Write unit tests for new functions
2. ✅ Add integration tests for new CLI features
3. ✅ Update test fixtures if detection rules change
4. ✅ Run `cargo test` before committing
5. ✅ Ensure `cargo clippy` passes
6. ✅ Update this document if adding new test categories

---

**Last Updated:** 2025-10-17
**Test Suite Version:** 1.0
**Total Tests:** 40 (30 active, 10 ignored)
