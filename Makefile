.PHONY: install-rust-tools
install-rust-tools:
	rustup update
	rustup component add rustfmt clippy

.PHONY: lint
lint:
	# Format files in the current crate using rustfmt
	cargo fmt -- --check
	# Check all packages and tests in the current crate and fail on warnings
	cargo clippy --all --tests -- --no-deps -D warnings

.PHONY: test
test:
	cargo test --no-fail-fast

.PHONY: publish
publish:
	cargo publish
