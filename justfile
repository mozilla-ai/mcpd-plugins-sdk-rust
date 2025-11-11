# justfile for mcpd-plugins-sdk-rust

# Default recipe to display help information.
default:
    @just --list

# Format code with rustfmt.
fmt:
    cargo fmt

# Check code formatting without making changes.
fmt-check:
    cargo fmt --check

# Run clippy linter.
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy with automatic fixes.
lint-fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty -- -D warnings

# Build the library.
build:
    cargo build

# Build in release mode.
release:
    cargo build --release

# Build static binary with musl (Linux only).
static:
    cargo build --release --target x86_64-unknown-linux-musl

# Run tests.
test:
    cargo test

# Run tests with output.
test-verbose:
    cargo test -- --nocapture

# Build all examples.
examples:
    cargo build --examples

# Build examples in release mode.
examples-release:
    cargo build --examples --release

# Run a specific example (usage: just run-example simple_plugin).
run-example name:
    cargo run --example {{name}} -- --address /tmp/{{name}}.sock

# Fast compile check.
check:
    cargo check --all-targets --all-features

# Check for security vulnerabilities.
audit:
    cargo audit

# Check dependencies with cargo-deny.
deny:
    cargo deny check

# Run all checks (format, lint, deny, audit, test).
ci: fmt-check lint deny audit test

# Clean build artifacts.
clean:
    cargo clean
    rm -rf proto/ src/generated/

# Generate documentation.
doc:
    cargo doc --no-deps --open

# Install required tools.
install-tools:
    rustup component add clippy rustfmt
    cargo install cargo-audit cargo-deny

# Install musl target for static builds.
install-musl:
    rustup target add x86_64-unknown-linux-musl

# Full pre-commit check.
pre-commit: fmt lint test

# Build everything (lib, examples, tests).
all: build examples test
