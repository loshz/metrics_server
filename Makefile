PUBLISH_FLAGS ?= --dry-run

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
	# https://doc.rust-lang.org/cargo/reference/publishing.html
	cargo package --list
	cargo publish ${PUBLISH_FLAGS}

.PHONY: gen-certs
gen-certs:
	mkdir -p ./tests/certs
	openssl genpkey -algorithm Ed25519 -out ./tests/certs/private_key.pem
	openssl req -x509 -key ./tests/certs/private_key.pem -out ./tests/certs/certificate.pem -sha256 -nodes -subj '/CN=localhost'
