# Release Notes: LLM-Guard v0.9.0

> **Release Focus** — Configurable input guardrail, hardened test coverage, refreshed rig workflows, and release tooling for the next tagged drop.

---

## 🎯 Overview

LLM-Guard v0.9.0 elevates the original hackathon prototype into a polished developer tool. Highlights include:

- A **configurable input-size guardrail** applied consistently across stdin, file, and tail mode (default 1 MB, override via `--max-input-bytes` or `LLM_GUARD_MAX_INPUT_BYTES`).
- **Expanded regression coverage** for tail-mode edge cases, input validation, and rig adapter fallbacks so providers that emit fenced or malformed JSON now fail safely.
- **Operator assets** such as branded example prompts and a maintainer-oriented `docs/RELEASE_CHECKLIST.md`.
- Documentation updates aligning with the current workspace (architecture, rule authoring, rig walkthrough) plus refreshed CLI integration tests demonstrating rig-backed providers end-to-end.

🎬 **Human roles:** architect, tester, and product manager partnered with GPT-5 Codex and Claude Code throughout this release.

---

## ✨ What’s New

### Guardrail & CLI Enhancements
- `--max-input-bytes` CLI flag and `LLM_GUARD_MAX_INPUT_BYTES` env var expose the input-size limit (default 1 MB).
- Tail mode streams through the same chunked UTF-8 reader, failing fast when the limit is exceeded and preserving snapshot integrity.
- All health checks can now run in dry-run mode; OpenAI profiles are validated without live API calls (ignored on macOS sandbox due to SystemConfiguration limits).

### Test Hardenings
- Added property/fuzz tests for tail behaviour, custom limits, and env/CLI precedence.
- Rig adapter regression tests cover fenced JSON payloads and ensure “unknown” fallbacks are emitted instead of panics.
- Keyword rule generation now filters out whitespace-only patterns, preventing spurious validation failures.

### Documentation & DX
- `docs/ARCHITECTURE.md` and `docs/RULE_AUTHORING.md` describe the workspace layout and rule pack conventions.
- `docs/RELEASE_CHECKLIST.md` codifies release prep (fmt/clippy/test, docs, tagging, announcements).
- `README.md` usage section reflects the guardrail flag, links to example prompts (`examples/prompt_safe.txt`, `prompt_suspicious.txt`, `prompt_malicious.txt`), and credits the human team roles.
- `docs/USAGE.md` now documents the new flag/env var and showcases rig-backed workflows with health checks.

### Planning Updates
- `PLAN.md` marks Phase 7–9 deliverables complete and defines the next steps: run the release checklist, groom stretch features (rule management, caching), and capture performance benchmarks.

---

## 🚀 Quick Start (unchanged)

```bash
git clone https://github.com/HendrikReh/llm-guard
cd llm-guard
cargo build --release

# Scan with optional guardrail override
./target/release/llm-guard-cli --max-input-bytes 2000000 scan --file examples/prompt_safe.txt

# Rig-backed scan with health pre-flight
./target/release/llm-guard-cli --providers-config llm_providers.yaml health --provider openai --dry-run
./target/release/llm-guard-cli scan --file examples/prompt_suspicious.txt --with-llm --provider openai
```

---

## 🧪 Testing

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
# Include ignored provider tests when network access is available:
cargo test -- --include-ignored
```

Tail fuzzer tests now run as part of the default suite; rig-related tests that require network/TLS remain ignored.

---

## 📚 Documentation Snapshot

| Document | Purpose |
|----------|---------|
| [`README.md`](./README.md) | Project overview, quick start, AI workflow insights |
| [`docs/USAGE.md`](./docs/USAGE.md) | CLI reference, global flags, advanced scenarios |
| [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) | End-to-end data flow and component responsibilities |
| [`docs/RULE_AUTHORING.md`](./docs/RULE_AUTHORING.md) | Extending keyword/regex packs safely |
| [`docs/RELEASE_CHECKLIST.md`](./docs/RELEASE_CHECKLIST.md) | Maintainer tasks before tagging a release |
| [`docs/TESTING_GUIDE.md`](./docs/TESTING_GUIDE.md) | Testing strategy, provider diagnostics |
| [`docs/SECURITY.md`](./docs/SECURITY.md) | Guardrails and data-handling assumptions |

---

## 📦 Packaging Checklist (Summary)

1. Run through `docs/RELEASE_CHECKLIST.md` (fmt/clippy/test, docs, version bumps).
2. Tag `v0.9.0`, push tags, and draft a release with binaries + changelog.
3. Share release summary with stakeholders (Slack/Discord/mailing list).

---

## 🔭 Next Steps

- Package the next tagged release using the new checklist.
- Groom stretch features: rule management commands, verdict caching, tail streaming optimisations.
- Capture performance benchmarks (latency, memory) across large prompts and document tuning guidance in `docs/ARCHITECTURE.md`.

---

## 📜 Historical Note — v0.4.1

The original hackathon release laid the foundation for LLM-Guard with multi-provider rig integration and core detection heuristics. Refer to the Git tag `v0.4.1` (or the GitHub Releases page) for the full historical changelog.
