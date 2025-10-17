# Security Guardrails

This document summarizes the hardening measures currently implemented in LLM-Guard and highlights the corresponding test coverage. Use it as a checklist when extending the scanner, CLI, or LLM adapters.

## Runtime Guardrails

- **Deterministic timeouts:** `LlmSettings::from_env` defaults to 30 second timeouts and exposes overrides via environment variables (`LLM_GUARD_TIMEOUT_SECS`). The CLI applies provider profiles before instantiating clients, ensuring every outbound call honours the configured timeout and retry budget.
- **Environment-sourced credentials:** API keys and other secrets are only read from the environment or provider profile files. The CLI never persists or echoes credentials back to stdout.
- **Strict linting:** Workspaces compile with `-D warnings` to surface potential misuse of unsafe APIs or unwraps in release builds.

## Scanner Safety

- **Length-normalised scoring:** `ScoreBreakdown::risk_score` clamps totals to the `[0, 100]` range, preventing overflow or runaway scores on long prompts. Property tests (`score_breakdown_never_exceeds_bounds`) ensure these clamps remain intact.
- **Family dampening:** Repeated rule hits are automatically dampened to reduce false positives from spammy prompts. Property tests (`family_dampening_caps_adjusted_total` and `scanning_repeated_instructions_remains_stable`) confirm the dampening logic holds across long inputs.
- **Excerpt redaction:** `extract_excerpt` truncates excerpts to `MAX_EXCERPT_CHARS` (240 characters) and sanitises UTF-8 boundaries. Property tests (`excerpt_limits_characters_and_boundaries`) guarantee the truncation never leaks more than the allowed window.
- **Finding validation:** The scanner validates every emitted `Finding` before returning a report, preventing downstream consumers from encountering invalid spans or weights.

## Testing & Tooling

- **Snapshot corpus:** Representative safe, suspicious, and malicious prompts live under `crates/llm-guard-core/tests/fixtures`. Snapshot tests (`scanner_snapshots.rs`) exercise the full rule pack and encode expected risk bands for regression detection.
- **Offline-friendly QA:** `cargo test` works in offline mode once dependencies are cached; CI runs `cargo nextest` and `cargo llvm-cov` to ensure reproducibility.
- **Future work:** Phase 7 tracks additional fuzzing around rule parsing and CLI input streaming. Any new guardrail should be documented here and paired with a regression test.

Keep this document in sync with the roadmap (`PLAN.md`) so that security posture improvements are traceable and test-backed.
