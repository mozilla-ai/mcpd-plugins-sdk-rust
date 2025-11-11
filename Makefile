# Makefile for mcpd-plugins-sdk-rust.
# Note: justfile is the recommended interface for development.

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  all            - Run fmt, lint, build, and test"
	@echo "  build          - Build the library"
	@echo "  release        - Build release version"
	@echo "  test           - Run tests"
	@echo "  clean          - Clean build artifacts"
	@echo "  examples       - Build all examples"
	@echo "  lint           - Run clippy linter"
	@echo "  fmt            - Format code"
	@echo "  check          - Fast compile check"
	@echo "  install-tools  - Install clippy and rustfmt"
	@echo "  run-simple     - Run simple_plugin example"
	@echo "  run-auth       - Run auth_plugin example"
	@echo "  run-ratelimit  - Run rate_limit_plugin example"
	@echo "  doc            - Build and open documentation"
	@echo "  coverage       - Generate test coverage"
	@echo "  static         - Build static binary (Linux musl)"

.PHONY: all
all: fmt lint build test

.PHONY: build
build:
	cargo build

.PHONY: release
release:
	cargo build --release

.PHONY: test
test:
	cargo test

.PHONY: clean
clean:
	cargo clean
	rm -rf proto/ src/generated/

.PHONY: examples
examples:
	cargo build --examples

.PHONY: lint
lint:
	cargo clippy -- -D warnings

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: check
check:
	cargo check

.PHONY: install-tools
install-tools:
	rustup component add clippy rustfmt

.PHONY: run-simple
run-simple:
	cargo run --example simple_plugin -- --address /tmp/simple.sock

.PHONY: run-auth
run-auth:
	cargo run --example auth_plugin -- --address /tmp/auth.sock

.PHONY: run-ratelimit
run-ratelimit:
	cargo run --example rate_limit_plugin -- --address /tmp/ratelimit.sock

.PHONY: doc
doc:
	cargo doc --no-deps --open

.PHONY: coverage
coverage:
	cargo tarpaulin --out Html --output-dir coverage/

.PHONY: static
static:
	cargo build --release --target x86_64-unknown-linux-musl
