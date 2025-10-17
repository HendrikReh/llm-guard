default: fmt lint test

fmt:
	cargo fmt

lint:
	cargo lint

test:
	if command -v cargo-nextest >/dev/null 2>&1; then \
		cargo nextest run --workspace --all-features --profile ci --failure-output=final; \
	else \
		cargo test --workspace --all-features; \
	fi

cov:
	cargo cov

audit:
	cargo audit

deny:
	cargo deny
