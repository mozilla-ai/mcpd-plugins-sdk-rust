# mcpd-plugins-sdk-rust

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

Rust SDK for building [mcpd](https://github.com/mozilla-ai/mcpd) plugins.

This SDK provides a simple, trait-based API for creating gRPC plugins that intercept and transform HTTP requests and responses in the mcpd middleware pipeline.

## Features

- **Simple trait-based API**: Implement the `Plugin` trait with only the methods you need
- **Async/await support**: Built on Tokio and Tonic for high-performance async I/O
- **Automatic server setup**: `serve()` function handles all boilerplate
- **Cross-platform**: Unix sockets (Linux/macOS) and TCP support
- **Type-safe**: Protocol buffers for serialization
- **Graceful shutdown**: SIGINT/SIGTERM handling with cleanup

## Quick Start

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
mcpd-plugins-sdk = "0.0" # any 0.0.x
tokio = { version = "1", features = ["full"] }
tonic = "0.12"
```

### Minimal Plugin Example

Create a simple plugin that adds a custom header:

```rust
use mcpd_plugins_sdk::{
    Plugin, serve, Metadata, Capabilities, HttpRequest, HttpResponse,
    FLOW_REQUEST,
};
use tonic::{Request, Response, Status};

struct MyPlugin;

#[tonic::async_trait]
impl Plugin for MyPlugin {
    async fn get_metadata(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Metadata>, Status> {
        Ok(Response::new(Metadata {
            name: "my-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "My custom plugin".to_string(),
            ..Default::default()
        }))
    }

    async fn get_capabilities(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Capabilities>, Status> {
        Ok(Response::new(Capabilities {
            flows: vec![FLOW_REQUEST as i32],
        }))
    }

    async fn handle_request(
        &self,
        request: Request<HttpRequest>,
    ) -> Result<Response<HttpResponse>, Status> {
        let mut req = request.into_inner();

        // Add custom header.
        req.headers.insert("X-My-Plugin".to_string(), "processed".to_string());

        Ok(Response::new(HttpResponse {
            r#continue: true,
            modified_request: Some(req),
            ..Default::default()
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    serve(MyPlugin, None).await?;
    Ok(())
}
```

### Building and Running

**Important**: When deploying plugins to run with mcpd (especially in containers), you should build **static, self-contained binaries** that don't depend on system libraries. The mcpd container won't have Rust runtime libraries available.

#### Recommended: Static Binary (Linux)

For production use with mcpd, build a fully static binary:

```bash
# Install musl target (one-time setup).
rustup target add x86_64-unknown-linux-musl

# Build static binary.
cargo build --release --target x86_64-unknown-linux-musl

# The resulting binary is completely self-contained and ready for mcpd.
./target/x86_64-unknown-linux-musl/release/my-plugin --address /tmp/my-plugin.sock
```

#### Development Build

For local development and testing:

```bash
# Build the plugin.
cargo build --release

# Run with Unix socket (Linux/macOS).
./target/release/my-plugin --address /tmp/my-plugin.sock --network unix

# Run with TCP (any platform).
./target/release/my-plugin --address localhost:50051 --network tcp
```

## Core Concepts

### Plugin Trait

The `Plugin` trait defines the interface for all plugins. All methods have default implementations, so you only need to override what you need:

```rust
#[tonic::async_trait]
pub trait Plugin: Send + Sync + 'static {
    // Identity methods.
    async fn get_metadata(&self, _request: Request<()>) -> Result<Response<Metadata>, Status>;
    async fn get_capabilities(&self, _request: Request<()>) -> Result<Response<Capabilities>, Status>;

    // Lifecycle methods.
    async fn configure(&self, _request: Request<PluginConfig>) -> Result<Response<()>, Status>;
    async fn stop(&self, _request: Request<()>) -> Result<Response<()>, Status>;

    // Health checks.
    async fn check_health(&self, _request: Request<()>) -> Result<Response<()>, Status>;
    async fn check_ready(&self, _request: Request<()>) -> Result<Response<()>, Status>;

    // Request/response handling.
    async fn handle_request(&self, request: Request<HttpRequest>) -> Result<Response<HttpResponse>, Status>;
    async fn handle_response(&self, response: Request<HttpResponse>) -> Result<Response<HttpResponse>, Status>;
}
```

### Processing Flows

Plugins can participate in two processing flows:

- **`FLOW_REQUEST`**: Process incoming HTTP requests before they reach the upstream server
- **`FLOW_RESPONSE`**: Process outgoing HTTP responses before they return to the client

Declare which flows your plugin supports in the `get_capabilities()` method:

```rust
async fn get_capabilities(&self, _request: Request<()>) -> Result<Response<Capabilities>, Status> {
    Ok(Response::new(Capabilities {
        flows: vec![FLOW_REQUEST as i32, FLOW_RESPONSE as i32],  // Support both flows.
    }))
}
```

### Request Handling Patterns

The `handle_request()` method can respond in three ways:

#### 1. Pass Through (Unchanged)

```rust
async fn handle_request(&self, request: Request<HttpRequest>) -> Result<Response<HttpResponse>, Status> {
    Ok(Response::new(HttpResponse {
        r#continue: true,
        ..Default::default()
    }))
}
```

#### 2. Transform Request

```rust
async fn handle_request(&self, request: Request<HttpRequest>) -> Result<Response<HttpResponse>, Status> {
    let mut req = request.into_inner();

    // Modify the request.
    req.headers.insert("X-Custom".to_string(), "value".to_string());

    Ok(Response::new(HttpResponse {
        r#continue: true,
        modified_request: Some(req),
        ..Default::default()
    }))
}
```

#### 3. Short-Circuit (Return Response)

```rust
async fn handle_request(&self, request: Request<HttpRequest>) -> Result<Response<HttpResponse>, Status> {
    // Return error response directly.
    Ok(Response::new(HttpResponse {
        r#continue: false,
        status_code: 401,
        body: b"Unauthorized".to_vec(),
        ..Default::default()
    }))
}
```

## Examples

The SDK includes three complete example plugins:

### 1. Simple Plugin

Adds custom headers to all requests.

```bash
cargo run --example simple_plugin -- --address /tmp/simple.sock
```

[View source](examples/simple_plugin/main.rs)

### 2. Auth Plugin

Validates Bearer tokens and returns 401 for unauthorized requests.

```bash
cargo run --example auth_plugin -- --address /tmp/auth.sock
```

[View source](examples/auth_plugin/main.rs)

### 3. Rate Limit Plugin

Implements token bucket rate limiting per client IP address.

```bash
cargo run --example rate_limit_plugin -- --address /tmp/ratelimit.sock
```

[View source](examples/rate_limit_plugin/main.rs)

## Building for Production

### Why Static Binaries?

mcpd runs plugins as separate processes and may run in containerized environments that don't have Rust runtime libraries. **Always use static binaries for production deployments.**

### Static Binary Build (Recommended)

```bash
# Install musl target (one-time setup).
rustup target add x86_64-unknown-linux-musl

# Build static binary.
cargo build --release --target x86_64-unknown-linux-musl

# The resulting binary is completely self-contained.
ls -lh target/x86_64-unknown-linux-musl/release/my-plugin
```

### Cross-Compilation

Use [cross](https://github.com/cross-rs/cross) for easy cross-compilation to different platforms:

```bash
# Install cross (one-time setup).
cargo install cross

# Build for different platforms.
cross build --release --target x86_64-unknown-linux-musl    # Linux x86_64 (static)
cross build --release --target aarch64-unknown-linux-musl   # Linux ARM64 (static)
cross build --release --target x86_64-apple-darwin          # macOS x86_64
cross build --release --target aarch64-apple-darwin         # macOS ARM64 (Apple Silicon)
```

### Binary Size Optimization

Add this to your plugin's `Cargo.toml` for smaller binaries (typically 3-5 MB):

```toml
[profile.release]
opt-level = "z"     # Optimize for size.
lto = true          # Enable link-time optimization.
codegen-units = 1   # Better optimization, slower compile.
strip = true        # Strip symbols.
panic = "abort"     # Smaller panic handler.
```

### Deployment Checklist

- ✅ Build with `--target x86_64-unknown-linux-musl` for static linking
- ✅ Test the binary runs without any dynamic library dependencies: `ldd my-plugin` (should show "not a dynamic executable")
- ✅ Verify the binary is executable and runs with `--help`
- ✅ Deploy to mcpd plugin directory with correct permissions

## Configuration

Plugins receive configuration via the `configure()` method:

```rust
async fn configure(&self, request: Request<PluginConfig>) -> Result<Response<()>, Status> {
    let config = request.into_inner();

    // Access custom configuration.
    if let Some(value) = config.custom_config.get("my_setting") {
        // Use configuration value.
    }

    // Access telemetry configuration.
    if let Some(telemetry) = config.telemetry {
        // Setup OpenTelemetry with provided settings.
    }

    Ok(Response::new(()))
}
```

Configuration is provided by mcpd from YAML files:

```yaml
plugins:
  my-plugin:
    custom_config:
      my_setting: "value"
      max_requests: "100"
```

## Error Handling

The SDK provides a `PluginError` type for error handling:

```rust
use mcpd_plugins_sdk::PluginError;

fn validate_config(value: &str) -> Result<u32, PluginError> {
    value.parse().map_err(|_| {
        PluginError::Configuration(format!("Invalid number: {}", value))
    })
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_request() {
        let plugin = MyPlugin::new();
        let request = Request::new(HttpRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            ..Default::default()
        });

        let response = plugin.handle_request(request).await.unwrap();
        assert_eq!(response.into_inner().r#continue, true);
    }
}
```

### Integration Tests

See the [examples](examples/) directory for complete integration test patterns.

## Protocol Buffers

The SDK automatically downloads and compiles protocol buffers from the [mcpd-proto](https://github.com/mozilla-ai/mcpd-proto) repository during the build process.

To use a specific proto version:

```bash
PROTO_VERSION=v0.0.3 cargo build
```

## Rust Version Policy

This crate requires Rust 1.75 or later. We follow a conservative MSRV policy and will clearly communicate any MSRV bumps.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Resources

- [mcpd](https://github.com/mozilla-ai/mcpd)
- [mcpd-proto](https://github.com/mozilla-ai/mcpd-proto) - Protocol buffer definitions
- [Tonic documentation](https://docs.rs/tonic/) - gRPC framework documentation
- [Tokio documentation](https://tokio.rs/) - Async runtime documentation

## Other Language SDKs

- [Go SDK](https://github.com/mozilla-ai/mcpd-plugins-sdk-go)
- [Python SDK](https://github.com/mozilla-ai/mcpd-plugins-sdk-python)
- [.NET SDK](https://github.com/mozilla-ai/mcpd-plugins-sdk-dotnet)
