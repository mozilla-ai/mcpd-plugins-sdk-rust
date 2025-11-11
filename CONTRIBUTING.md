# Contributing to mcpd-plugins-sdk-rust

Thank you for your interest in contributing to the Rust SDK for mcpd plugins!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/mcpd-plugins-sdk-rust`
3. Create a feature branch: `git checkout -b my-feature`
4. Make your changes
5. Run tests and linting: `make test lint`
6. Commit your changes
7. Push to your fork: `git push origin my-feature`
8. Open a pull request

## Development Setup

### Prerequisites

- Rust 1.75 or later
- `cargo` and `rustup`

### Install Development Tools

```bash
make install-tools
```

This installs:
- `clippy` - Rust linter
- `rustfmt` - Code formatter

### Building

```bash
# Build the library.
make build

# Build examples.
make examples

# Build everything.
make all
```

### Testing

```bash
# Run all tests.
make test

# Run specific test.
cargo test test_name
```

### Linting and Formatting

We use `clippy` and `rustfmt` to maintain code quality:

```bash
# Run linter.
make lint

# Format code.
make fmt

# Both are run automatically by `make all`.
```

## Code Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format your code
- Address all `clippy` warnings
- Add documentation comments for public items
- End single-line comments with a period

### Documentation

All public items must have documentation comments:

```rust
/// Returns plugin metadata.
///
/// This method should be overridden to provide plugin identification.
async fn get_metadata(&self, _request: Request<()>) -> Result<Response<Metadata>, Status> {
    // Implementation.
}
```

## Testing Guidelines

- Write unit tests for new functionality
- Add integration tests for examples
- Ensure all tests pass before submitting PR
- Use descriptive test names

## Pull Request Process

1. Update documentation if you've changed APIs
2. Add tests for new functionality
3. Ensure `make all` passes without errors
4. Update CHANGELOG.md if applicable
5. Write clear commit messages
6. Reference any related issues in the PR description

## Commit Messages

- Use clear, descriptive commit messages
- Reference issues: "Fix #123: Description"
- Use present tense: "Add feature" not "Added feature"

## Code Review

- All PRs require review before merging
- Address review feedback promptly
- Keep PRs focused on a single feature/fix

## Questions?

Open an issue for questions or discussion.

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
