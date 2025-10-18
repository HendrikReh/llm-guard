# Rule Authoring Guide

This guide explains how to extend LLM-Guard’s heuristic rule packs. The scanner consumes two files found under the `rules/` directory:

- `keywords.txt` — simple literal matches parsed via Aho-Corasick
- `patterns.json` — regular-expression rules compiled with `regex`

## Keyword Rules (`keywords.txt`)

Each non-comment line uses a pipe-delimited format:

```
RULE_ID|WEIGHT|Description shown in reports|pattern text
```

Example:

```
INSTR_OVERRIDE|25|Attempts to override instructions|ignore previous
```

Guidelines:

- **ID** — Uppercase with underscores, grouped by family (`INSTR`, `CODE`, `DATA`, …). Families drive score dampening.
- **Weight** — Float between `0.0` and `100.0`. Use higher weights for high-risk indicators; related rules should share similar scales.
- **Description** — Keep concise; it appears verbatim in human reports.
- **Pattern** — Literal substring. Case-sensitive by default; include both lowercase/uppercase variants if needed.
- **Comments** — Lines beginning with `#` are ignored.

## Regex Rules (`patterns.json`)

`patterns.json` is a JSON array of rule objects:

```json
[
  {
    "id": "CODE_SHELL",
    "description": "Attempts to execute shell commands",
    "pattern": "run\\s+bash",
    "weight": 50.0,
    "window": 64
  }
]
```

Fields:

- `id`, `description`, `weight` — Same conventions as keyword rules.
- `pattern` — Rust `regex` syntax; remember to double-escape backslashes.
- `window` *(optional)* — Extra characters of context to capture on either side of the match (defaults to 64). Set only when added context is useful in reports.

## Validation & Testing

Automated guards prevent malformed packs:

- Duplicate IDs are rejected.
- Weights must remain within `0.0..=100.0`.
- Regex patterns must compile.
- `window` must be greater than zero when provided.

After editing rule files, run:

```bash
cargo test -p llm-guard-core -- scanner::file_repository
```

or the full suite:

```bash
cargo test
```

Add new regression fixtures under `tests/` if rules target novel behaviours. Snapshot tests in `tests/scanner_snapshots.rs` demonstrate how to capture expected risk scores.

## Assigning Weights

| Impact | Suggested Weight | Notes |
|--------|------------------|-------|
| Minor policy nudge | 5–15 | e.g., polite attempts to see the system prompt |
| Clear guardrail bypass | 20–40 | direct “ignore previous instructions” patterns |
| High-risk execution | 40–70 | code execution, credential exfiltration |
| Catastrophic | 70–100 | indiscriminate data leakage, direct malware instructions |

Combine multiple lower-weight rules if a single indicator is too noisy. The family dampening factor (default `0.5`) halves the weight for repeated matches in the same family beyond the first hit.

## Shipping Custom Packs

1. Edit `rules/keywords.txt` and/or `rules/patterns.json`.
2. Document the changes in release notes or a dedicated rule changelog.
3. Share updated packs with users; `--rules-dir` lets operators point to alternate directories.
4. Consider adding integration tests that scan representative prompts from your domain.

## See Also

- [ARCHITECTURE.md](./ARCHITECTURE.md)
- [docs/USAGE.md](./USAGE.md)
- [ADR 0004 — Aho-Corasick & Regex Detection](./ADR/0004-aho-corasick-regex-detection.md)
