# LLM-Guard — Prompt Injection Firewall

A fast, explainable **Rust** CLI that scans prompts/logs for **prompt-injection & jailbreak indicators**, scores the risk (0–100), and optionally asks an LLM (e.g., Codex) for a short verdict and mitigation tip.

> Any descriptions of LLM behavior are based on observed patterns and may vary.

## Why this exists

- **Hackathon-ready:** built to be implemented in a single day with vibe-coding loops.  
- **Explainable security:** rule hits + weights + excerpts → transparent scoring.  
- **CI friendly:** JSON output for pipelines; human-readable, colorized CLI for demos.


## Quickstart

```bash
# 1) Create project (if not already cloned)
git clone https://github.com/yourname/llm-guard
cd llm-guard

# 2) Build
cargo build --release

# 3) Run on a file
./target/release/llm-guard scan --file samples/chat.txt

# 4) JSON for CI
./target/release/llm-guard scan --file samples/chat.txt --json > report.json

# 5) Optional: LLM verdict (requires API key in env)
export OPENAI_API_KEY=...   # or your provider
./target/release/llm-guard scan --file samples/chat.txt --with-llm
```

## What it does

- Loads **keyword** and **regex** rules (Aho-Corasick + `regex`).
- Scans input (stdin/file/stream) and emits **findings** (rule, span, excerpt, weight).
- Computes a **risk score** with length normalization & synergy bonuses.
- (Optional) Queries an LLM for **label + rationale + mitigation** (short JSON).

## Hackathon Context

Developed during an **AI Coding Hackathon** as part of the [AI Coding Accelerator (Maven)](https://maven.com/nila/ai-coding-accelerator) course.

**Instructors:** Vignesh Mohankumar and Jason Liu

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

**Core crates:** `clap`, `regex`, `aho-corasick`, `serde`, `serde_json`, `anyhow`, `colored`/`ansi_term`, `once_cell`, `humantime`  
**Optional:** `notify`, `comfy-table`, `tokio`

## License

MIT — see `LICENSE`.

## ⚠️ Disclaimer

This tool uses heuristic rules and optionally an LLM for guidance.  
It **does not guarantee** prevention of all prompt-injection attempts; treat scores as decision support, not absolute truth.
