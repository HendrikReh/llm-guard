default: fmt lint test

fmt:
	cargo fmt

lint:
	cargo lint

test:
	cargo test-all

cov:
	cargo cov

audit:
	cargo audit

deny:
	cargo deny
