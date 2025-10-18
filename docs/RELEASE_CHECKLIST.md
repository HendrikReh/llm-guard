# Release Checklist

Use this checklist when preparing a tagged release of LLM-Guard.

## Pre-Release Verification

- [ ] Update `PLAN.md` and `RELEASE_NOTES.md` with the upcoming version.
- [ ] Confirm `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` pass locally.
- [ ] Ensure integration tests (`tests/scan_llm.rs`, `tests/health.rs`) reflect the current CLI usage.
- [ ] Verify rule packs (`rules/keywords.txt`, `rules/patterns.json`) parse with `cargo test -p llm-guard-core`.
- [ ] Regenerate documentation snippets if changed (README usage examples, docs/USAGE.md).
- [ ] Manually scan the example prompts under `examples/` and capture sample output.
- [ ] Update version numbers in `Cargo.toml` (workspace and crates) if tagging a new release.

## Packaging

- [ ] Run `cargo build --release` and smoke test `target/release/llm-guard-cli`.
- [ ] Produce a checksum for the release binary when distributing artifacts.
- [ ] Attach `docs/RELEASE_NOTES.md` summary to the GitHub release draft.

## Post-Release

- [ ] Tag the release (`git tag vX.Y.Z && git push --tags`).
- [ ] Publish announcement/update (Slack, email, or community channels).
- [ ] Monitor issue tracker for regressions or release-specific bugs.
