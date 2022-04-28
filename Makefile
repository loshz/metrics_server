PUBLISH_FLAGS ?= --dry-run

.PHONY: install-rust-tools audit lint test publish gen-certs

install-rust-tools:
	rustup update
	rustup component add rustfmt clippy
	cargo install --locked cargo-deny

audit:
	cargo deny check

lint:
	# Format files in the current crate using rustfmt
	cargo fmt -- --check
	# Check all packages and tests in the current crate and fail on warnings
	cargo clippy --all --tests -- --no-deps -D warnings

test:
	cargo test --no-fail-fast

publish:
	# https://doc.rust-lang.org/cargo/reference/publishing.html
	cargo package --list
	cargo publish ${PUBLISH_FLAGS}

gen-certs:
	mkdir -p ./tests/certs
	openssl genpkey -algorithm Ed25519 -out ./tests/certs/private_key.pem
	openssl req -x509 -key ./tests/certs/private_key.pem -out ./tests/certs/certificate.pem -sha256 -nodes -subj '/CN=localhost'
