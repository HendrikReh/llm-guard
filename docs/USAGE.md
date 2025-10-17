# LLM-Guard CLI Usage Guide

> **AI-Assisted Development Note:** This CLI was built in ~7 hours using GPT-5 Codex and Claude Code. See [README.md](../README.md) for workflow insights.

Comprehensive reference for all `llm-guard` commands, flags, and configuration options.

## Table of Contents

- [Global Options](#global-options)
- [Commands](#commands)
  - [`list-rules`](#list-rules)
  - [`scan`](#scan)
  - [`health`](#health)
- [Configuration Sources](#configuration-sources)
  - [Configuration Precedence](#configuration-precedence)
  - [Environment Variables](#environment-variables)
  - [Provider Profiles](#provider-profiles)
- [Exit Codes](#exit-codes)
- [Examples](#examples)
  - [Basic Usage](#basic-usage)
  - [LLM Integration](#llm-integration)
  - [Health Checks](#health-checks)
  - [Advanced Scenarios](#advanced-scenarios)
- [Related Documentation](#related-documentation)

---

## Global Options

Global flags apply to all commands and must be specified **before** the subcommand.

| Flag | Description | Default |
| ---- | ----------- | ------- |
| `--rules-dir <DIR>` | Directory containing rule packs (`keywords.txt`, `patterns.json`) | `./rules` |
| `--config <FILE>` | Application config file (TOML/YAML/JSON) | _none_ |
| `--providers-config <FILE>` | YAML file with per-provider credentials and settings | `llm_providers.yaml` |
| `--debug` | Enable verbose diagnostics; logs raw provider payloads on parse errors | `false` |
| `--help`, `-h` | Display help text | - |
| `--version`, `-V` | Print CLI version | - |

**Example:**
```bash
llm-guard --debug --rules-dir /etc/llm-guard/rules scan --file prompt.txt
```

> **Tip:** Global options can also be set via environment variables or config files. See [Configuration Sources](#configuration-sources).

---

## Commands

### `list-rules`

Display all detection rules currently loaded from the rules directory.

**Usage:**
```bash
llm-guard list-rules [OPTIONS]
```

**Options:**

| Flag | Description | Default |
| ---- | ----------- | ------- |
| `--json` | Output rules as JSON array | `false` (human-readable) |

**Example Output (Human-Readable):**
```
Rule ID: INSTR_OVERRIDE
  Pattern: ignore previous instructions
  Weight: 16.0
  Family: instruction_override
  Description: Attempts to override system instructions

Rule ID: PROMPT_LEAK
  Pattern: reveal system prompt
  Weight: 14.0
  Family: data_exfiltration
  Description: Attempts to leak system prompts
```

**Example Output (JSON):**
```json
[
  {
    "id": "INSTR_OVERRIDE",
    "pattern": "ignore previous instructions",
    "weight": 16.0,
    "family": "instruction_override",
    "description": "Attempts to override system instructions"
  }
]
```

### `scan`

Run the detection engine against stdin or a file, optionally enriched with LLM analysis.

**Usage:**
```bash
llm-guard scan [OPTIONS]
```

**Options:**

| Flag | Description | Default |
| ---- | ----------- | ------- |
| `--file <PATH>` | Input file to scan | stdin |
| `--json` | Output JSON report | `false` (human-readable) |
| `--tail` | Tail file and rescan on changes (requires `--file`) | `false` |
| `--with-llm` | Add LLM verdict to heuristic report | `false` |

**LLM Provider Overrides:**

| Flag | Description | Example |
| ---- | ----------- | ------- |
| `--provider <NAME>` | Provider: `openai`, `anthropic`, `gemini`, `azure`, `noop` | `anthropic` |
| `--model <MODEL>` | Model identifier | `gpt-4o-mini`, `claude-3-5-haiku-20241022` |
| `--endpoint <URL>` | Custom provider base URL | `https://api.openai.com` |
| `--deployment <NAME>` | Azure OpenAI deployment name | `gpt-4o-production` |
| `--project <NAME>` | Provider project ID (Gemini, Anthropic) | `security-project` |
| `--workspace <NAME>` | Provider workspace ID | `default` |

**Exit Codes:**
- `0` — Low risk (score < 25)
- `2` — Medium risk (score 25-59)
- `3` — High risk (score ≥ 60)
- `1` — Error (file not found, parse failure, etc.)

#### Streaming Tail Mode

- `--tail` polls the target file every two seconds (configurable via `tail_file` in tests) and only re-scans when the contents change.
- Each refresh prints a banner with the file path followed by the rendered report (respecting `--json`).
- The tail loop is fuzz-tested to ensure rapid updates or alternating prompt content do not panic and always return the final risk band exit code.

**Example Output (Human-Readable):**
```
Risk: 72/100  (HIGH)

Findings:
  [INSTR_OVERRIDE] "ignore previous instructions" at 0..29  (+16)
  [PROMPT_LEAK]    "reveal system prompt" at 45..65        (+14)

Synergy bonus (override+leak within 200 chars)              (+5)

LLM Verdict: malicious
  Rationale: Prompt combines instruction override with data exfiltration
  Mitigation: Reject prompt and log for security review
```

**Example Output (JSON):**
```json
{
  "risk_score": 72.0,
  "risk_band": "high",
  "findings": [
    {
      "rule_id": "INSTR_OVERRIDE",
      "excerpt": "ignore previous instructions",
      "span": [0, 29],
      "weight": 16.0
    }
  ],
  "llm_verdict": {
    "classification": "malicious",
    "rationale": "Prompt combines instruction override with data exfiltration",
    "mitigation": "Reject prompt and log for security review"
  }
}
```

### `health`

Validate LLM provider configuration and connectivity with optional live API calls.

**Usage:**
```bash
llm-guard health [OPTIONS]
```

**Options:**

| Flag | Description | Default |
| ---- | ----------- | ------- |
| `--provider <NAME>` | Check specific provider only | all configured providers |
| `--dry-run` | Skip live API calls; validate config only | `false` |

**Exit Codes:**
- `0` — All checks passed
- `1` — Configuration errors or API failures

**Use Cases:**
- **CI/CD:** Validate provider credentials in deployment pipelines
- **Debugging:** Test provider connectivity before scanning
- **Smoke Tests:** Verify multi-provider setup after configuration changes

#### Rig-Backed Examples

```bash
# Use provider profiles (llm_providers.yaml) and add rig-powered verdicts
llm-guard scan --file samples/chat.txt --with-llm

# Override provider/model inline (falls back to rig.rs OpenAI adapter)
llm-guard scan --file samples/chat.txt --with-llm \
  --provider openai \
  --model gpt-4o-mini \
  --endpoint https://api.openai.com

# Target Anthropic via rig.rs with a custom project
llm-guard scan --file samples/chat.txt --with-llm \
  --provider anthropic \
  --project security-research
```

**Example Output:**
```
Provider: openai
  ✓ Configuration valid
  ✓ API connectivity confirmed
  ✓ Model: gpt-4o-mini

Provider: anthropic
  ✓ Configuration valid
  ✓ API connectivity confirmed
  ✓ Model: claude-3-5-haiku-20241022

All health checks passed.
```

**Dry-Run Example:**
```bash
# Validate configuration without API calls
llm-guard health --dry-run

Provider: openai
  ✓ Configuration valid (dry-run)
Provider: anthropic
  ✓ Configuration valid (dry-run)
```

---

## Configuration Sources

LLM-Guard supports multiple configuration methods with clear precedence rules.

### Configuration Precedence

Settings are resolved in this order (highest priority first):

1. **CLI Flags** — Command-line arguments override everything
2. **Environment Variables** — `LLM_GUARD_*` variables
3. **Provider Profiles** — `llm_providers.yaml` (or `--providers-config`)
4. **Application Config** — TOML/YAML/JSON file via `--config`
5. **Built-in Defaults** — `openai`, `gpt-4o-mini`, `max_retries=2`

### Environment Variables

All LLM settings can be configured via environment variables:

| Variable | Description | Example |
| -------- | ----------- | ------- |
| `LLM_GUARD_PROVIDER` | Provider identifier | `openai`, `anthropic`, `gemini`, `azure` |
| `LLM_GUARD_API_KEY` | API key or token | `sk-...` |
| `LLM_GUARD_ENDPOINT` | Custom base URL | `https://api.openai.com` |
| `LLM_GUARD_MODEL` | Model name | `gpt-4o-mini` |
| `LLM_GUARD_DEPLOYMENT` | Azure deployment name | `gpt-4o-production` |
| `LLM_GUARD_PROJECT` | Provider project ID | `security-project` |
| `LLM_GUARD_WORKSPACE` | Provider workspace | `default` |
| `LLM_GUARD_TIMEOUT_SECS` | HTTP timeout | `30` |
| `LLM_GUARD_MAX_RETRIES` | Retry count | `2` |
| `LLM_GUARD_API_VERSION` | API version (Azure) | `2024-02-15-preview` |
| `LLM_GUARD_DEBUG` | Enable debug logging | `1` |

**Example:**
```bash
export LLM_GUARD_PROVIDER=anthropic
export LLM_GUARD_API_KEY=sk-ant-...
export LLM_GUARD_MODEL=claude-3-5-haiku-20241022
llm-guard scan --file prompt.txt --with-llm
```

### Provider Profiles

The `llm_providers.yaml` file allows managing multiple providers simultaneously:

```yaml
providers:
  - name: "openai"
    api_key: "sk-..."
    model: "gpt-4o-mini"

  - name: "anthropic"
    api_key: "sk-ant-..."
    model: "claude-3-5-haiku-20241022"

  - name: "azure"
    api_key: "..."
    endpoint: "https://your-resource.openai.azure.com"
    deployment: "gpt-4o-production"
    api_version: "2024-02-15-preview"
    timeout_secs: 60
    max_retries: 3

  - name: "gemini"
    api_key: "..."
    project: "security-project"
```

**Benefits:**
- Store credentials for multiple providers
- Switch providers with `--provider` flag
- Version-controlled defaults (use `.gitignore` for secrets)
- Team-wide configuration consistency

**Usage:**
```bash
# Use OpenAI (first in file)
llm-guard scan --file prompt.txt --with-llm

# Switch to Anthropic
llm-guard scan --file prompt.txt --with-llm --provider anthropic

# Custom config location
llm-guard --providers-config /etc/llm-guard/providers.yaml scan --with-llm
```

---

## Exit Codes

LLM-Guard uses meaningful exit codes for CI/CD integration:

| Code | Meaning | Use Case |
| ---- | ------- | -------- |
| `0` | Success (low risk: score < 25) | Allow request to proceed |
| `2` | Medium risk (score 25-59) | Flag for human review |
| `3` | High risk (score ≥ 60) | Block request immediately |
| `1` | Error (file not found, config invalid, etc.) | Fix configuration or input |

**CI/CD Example:**
```bash
#!/bin/bash
llm-guard scan --file user_prompt.txt --json > report.json
EXIT_CODE=$?

if [ $EXIT_CODE -eq 3 ]; then
  echo "High risk detected! Blocking request."
  exit 1
elif [ $EXIT_CODE -eq 2 ]; then
  echo "Medium risk. Manual review required."
  # Trigger review workflow
elif [ $EXIT_CODE -eq 0 ]; then
  echo "Safe to proceed."
else
  echo "Error occurred. Check configuration."
  exit 1
fi
```

---

## Examples

### Basic Usage

```bash
# List all detection rules
llm-guard list-rules

# List rules as JSON
llm-guard list-rules --json

# Scan from stdin
cat prompt.txt | llm-guard scan

# Scan a file
llm-guard scan --file samples/chat.txt

# Scan with JSON output
llm-guard scan --file samples/chat.txt --json

# Tail and scan log file continuously
llm-guard scan --file logs/chat.log --tail
```

### LLM Integration

```bash
# Scan with LLM verdict (uses config/env)
llm-guard scan --file samples/chat.txt --with-llm

# Override provider on-the-fly
llm-guard scan --file samples/chat.txt --with-llm \
  --provider anthropic --model claude-3-5-haiku-20241022

# Use Azure OpenAI
llm-guard scan --file samples/chat.txt --with-llm \
  --provider azure \
  --deployment gpt-4o-production \
  --endpoint https://your-resource.openai.azure.com

# Dry-run with noop provider (no API calls)
llm-guard scan --file samples/chat.txt --with-llm --provider noop

# Debug mode (log raw provider responses)
llm-guard --debug scan --file samples/chat.txt --with-llm
```

### Health Checks

```bash
# Check all configured providers
llm-guard health --providers-config llm_providers.yaml

# Check specific provider
llm-guard health --provider anthropic

# Dry-run (validate config without API calls)
llm-guard health --dry-run

# Debug health check issues
llm-guard --debug health --provider openai
```

### Advanced Scenarios

```bash
# Custom rules directory
llm-guard --rules-dir /etc/llm-guard/rules scan --file prompt.txt

# Multi-provider testing
for provider in openai anthropic gemini; do
  echo "Testing $provider..."
  llm-guard scan --file samples/chat.txt --with-llm --provider $provider
done

# CI/CD integration with exit codes
llm-guard scan --file user_input.txt --json > report.json
if [ $? -ge 2 ]; then
  echo "Risk detected! See report.json"
  exit 1
fi

# Streaming log analysis with LLM
llm-guard scan --file /var/log/chatbot.log --tail --with-llm --json | \
  jq 'select(.risk_score > 50)'
```

---

## Rig Provider Walkthrough

1. **Create provider profile:** Add entries to `llm_providers.yaml`, for example:

   ```yaml
   providers:
     - name: openai
       api_key: ${OPENAI_API_KEY}
     - name: anthropic
       api_key: ${ANTHROPIC_API_KEY}
       project: security-research
   ```

2. **Prime the environment:** Profiles are auto-loaded when `--with-llm` is used. CLI flags (`--model`, `--project`, `--endpoint`, `--workspace`) take precedence if supplied.

3. **Run the scan:** `llm-guard scan --file samples/chat.txt --with-llm` will ask the rig-backed provider for a JSON verdict and merge it into the heuristic report.

4. **Inspect streaming mode:** Combine `--tail` with `--with-llm` to watch long-running logs. The tail loop is fuzz-tested to handle rapid file mutations without panicking.

> Screenshots used in demos: `docs/screenshots/rig-openai.png` and `docs/screenshots/rig-anthropic.png` (**not version-controlled**; skip if unavailable).

---

## Rig Provider Troubleshooting

- **`API key must be provided`** — Ensure `LLM_GUARD_API_KEY` (or provider profile entry) is set and non-empty. Noop provider is the only exception.
- **`requires endpoint` (Azure)** — Supply `--endpoint` or `LLM_GUARD_ENDPOINT` pointing to your Azure resource (e.g., `https://example.openai.azure.com`).
- **`requires deployment`** — Provide `--deployment`/`LLM_GUARD_DEPLOYMENT` or reuse `--model` as the deployment name.
- **Empty or non-JSON verdicts** — Enable `--debug` to log raw provider responses; the rig adapter will fall back to an "unknown" verdict rather than panic.
- **HTTP 401/403** — Regenerate API keys or confirm tenant/project values (Anthropic/Gemini frequently require project/workspace settings).

---

## Related Documentation

- **[README.md](../README.md)** — Project overview, features, and AI workflow insights
- **[TESTING_GUIDE.md](./TESTING_GUIDE.md)** — Testing strategies and commands
- **[PRD.md](../PRD.md)** — Product requirements and technical specifications
- **[AGENTS.md](../AGENTS.md)** — AI coding assistant onboarding guide
- **[docs/ADR/](./ADR/)** — Architecture decision records

---

**Last Updated:** 2025-10-17
