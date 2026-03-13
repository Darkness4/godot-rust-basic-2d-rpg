.PHONY: lint
lint:
	cargo fmt --all -- --check
	cargo clippy -- -D warnings

.PHONY: fix
fix:
	cargo fmt --all
	cargo clippy --fix --allow-dirty
