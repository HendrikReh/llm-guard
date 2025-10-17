# LLM-Guard — Prompt Injection Firewall

A fast, explainable **Rust** CLI that scans prompts and logs for **prompt-injection & jailbreak indicators**, scores the risk (0–100), and optionally asks an LLM for a short verdict and mitigation tip.

> **Note:** Any descriptions of LLM behavior are based on observed patterns and may vary.

## Status

LLM-Guard is under active development. See `PLAN.md` for the detailed implementation roadmap and progress tracking.

## Features

- **Explainable Security:** Transparent scoring with rule hits, weights, and text excerpts
- **Fast & Lightweight:** Built in Rust with Aho-Corasick and optimized regex matching
- **CI/CD Friendly:** JSON output for automated pipelines; colorized CLI for interactive use
- **Optional LLM Analysis:** Get additional context and mitigation suggestions from language models
- **Flexible Input:** Scan files, stdin, or streaming logs


## Installation

```bash
git clone https://github.com/yourname/llm-guard
cd llm-guard
cargo build --release
```

## Usage

### Basic Scanning

```bash
# Scan a file
./target/release/llm-guard scan --file samples/chat.txt

# Scan from stdin
echo "Ignore previous instructions" | ./target/release/llm-guard scan
```

### CI/CD Integration

```bash
# Generate JSON output for automated processing
./target/release/llm-guard scan --file samples/chat.txt --json > report.json

# Tail a log file and rescan on change (Ctrl+C to stop)
./target/release/llm-guard scan --file logs/chat.log --tail
```

### With LLM Analysis

```bash
# Set your API key
export LLM_GUARD_API_KEY=your_key_here
# Optional provider/endpoint overrides
export LLM_GUARD_PROVIDER=openai
export LLM_GUARD_ENDPOINT=https://api.openai.com/v1

# (Placeholder) Request additional LLM-powered analysis
./target/release/llm-guard scan --file samples/chat.txt --with-llm

```

> `--with-llm` is reserved for Phase 6 and will return an error until the adapter is implemented.

### LLM Configuration (preview)

Environment variables expected by future LLM adapters:

- `LLM_GUARD_PROVIDER` — provider identifier (default: `openai`).
- `LLM_GUARD_API_KEY` — required API key/token.
- `LLM_GUARD_ENDPOINT` — optional custom endpoint/base URL.

### Exit Codes

The `scan` command returns an exit status that reflects the highest risk band encountered:

- `0` — Low risk (informational)
- `2` — Medium risk (requires review / mitigation)
- `3` — High risk (block or re-prompt)
- `1` — CLI error (I/O failure, invalid flag, etc.)
```

## How It Works

1. **Rule Loading:** Loads keyword and regex-based detection rules using Aho-Corasick and `regex` crate
2. **Scanning:** Processes input from stdin, files, or streams to identify potential threats
3. **Finding Generation:** Emits detailed findings with rule matches, text spans, excerpts, and weights
4. **Risk Scoring:** Computes a normalized risk score (0–100) with length normalization and synergy bonuses
5. **LLM Analysis (Optional):** Queries an LLM for classification, rationale, and mitigation suggestions

## Architecture

```
cli (clap)
  ├─ reader (stdin | file | tail -f)
  ├─ scanner
  │    ├─ rules (regex + aho-corasick)
  │    ├─ heuristics (weights, windows, caps)
  │    └─ explain (feature attributions)
  ├─ llm_adapter (optional)
  └─ reporters (human / json)
```

### Dependencies

**Core:** `clap`, `regex`, `aho-corasick`, `serde`, `serde_json`, `anyhow`, `once_cell`, `humantime`, `tokio`, `tracing`

## About

Developed during an **AI Coding Hackathon** as part of the [AI Coding Accelerator (Maven)](https://maven.com/nila/ai-coding-accelerator) course.

**Instructors:** Vignesh Mohankumar and Jason Liu

## License

MIT — see `LICENSE`.

## Disclaimer

This tool uses heuristic rules and optional LLM analysis for threat detection. It **does not guarantee** prevention of all prompt-injection attempts. Treat risk scores as decision support, not absolute truth. Use as part of a defense-in-depth security strategy.
