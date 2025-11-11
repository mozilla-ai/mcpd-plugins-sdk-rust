# Architecture Documentation

## Overview

The `mcpd-plugins-sdk-rust` provides a Rust implementation of the `mcpd` plugin SDK, following the same patterns established in the [Go](https://github.com/mozilla-ai/mcpd-plugins-sdk-go), [Python](https://github.com/mozilla-ai/mcpd-plugins-sdk-python), and [.NET](https://github.com/mozilla-ai/mcpd-plugins-sdk-dotnet) SDKs while adhering to Rust idioms and best practices.

## Design Principles

### 1. Trait-Based Architecture

Unlike the other SDKs which use class inheritance, this Rust SDK uses a trait-based approach:

- **`Plugin` trait**: Core trait that plugins implement
- **Default implementations**: All methods have sensible defaults
- **Zero-cost abstractions**: Trait dispatch optimized by the compiler

### 2. Async-First Design

Following Rust async best practices:

- All plugin methods are `async` using `#[tonic::async_trait]`
- Built on Tokio for high-performance async I/O
- Graceful shutdown with signal handling

### 3. Type Safety

- Protocol buffers generated at build time
- Strong typing throughout the API
- Custom error types with `thiserror`


## Module Structure

```
src/
├── generated/      - Generated protobuf code
├── constants.rs    - Flow constants
├── error.rs        - Error types
├── lib.rs          - Public API exports and documentation
├── plugin.rs       - Plugin trait and adapter
└── server.rs       - Server lifecycle management
```

### generated/

Auto-generated protobuf and gRPC code:
- Downloaded from mcpd-proto repository at build time
- Compiled from [`plugin.proto`](https://github.com/mozilla-ai/mcpd-proto/blob/main/plugins/v1/plugin.proto) using tonic-build
- Contains message types and service traits

### constants.rs

Flow constants for plugin capabilities:
- `FLOW_REQUEST` - Process incoming HTTP requests
- `FLOW_RESPONSE` - Process outgoing HTTP responses

### error.rs

Custom error types:
- `PluginError` enum with variants for different error types
- Conversion to gRPC `Status` codes
- Integration with `std::error::Error`

### lib.rs

The main entry point that:
- Re-exports public API types
- Provides comprehensive documentation
- Includes generated protobuf module

### plugin.rs

Defines the core `Plugin` trait with:
- 8 methods: metadata, capabilities, lifecycle, health, and request handling
- Default implementations for all methods
- `PluginAdapter` to bridge between trait and generated gRPC service

### server.rs

Handles server lifecycle:
- Command-line argument parsing with `clap`
- Unix socket and TCP support
- Graceful shutdown with signal handling
- Automatic socket cleanup

## Key Design Decisions

### 1. Raw String Literals for Reserved Keywords

The protobuf field `continue` is a Rust keyword, so we use `r#continue`:

```rust
HttpResponse {
    r#continue: true,
    ..Default::default()
}
```

### 2. Build-Time Proto Download

The `build.rs` script:
- Downloads proto files from `mcpd-proto` [repository](https://github.com/mozilla-ai/mcpd-proto)
- Generates Rust code with `tonic-build`
- Ensures reproducible builds

### 3. Cross-Platform Support

- Unix sockets on Linux/macOS (preferred)
- TCP on all platforms (including Windows)
- Conditional compilation with `#[cfg(unix)]`

### 4. Zero-Copy Where Possible

- References instead of clones
- `Arc` for shared ownership
- Borrowed data in function parameters

## Comparison with Other SDKs

### Go SDK

https://github.com/mozilla-ai/mcpd-plugins-sdk-go

- **Go**: Struct embedding for composition
- **Rust**: Trait implementation

### Python SDK

https://github.com/mozilla-ai/mcpd-plugins-sdk-python

- **Python**: Class inheritance with `async/await`
- **Rust**: Trait implementation with `#[tonic::async_trait]`

### .NET SDK

https://github.com/mozilla-ai/mcpd-plugins-sdk-dotnet

- **C#**: Class inheritance with `Task<T>`
- **Rust**: Trait implementation with `async fn`

## Performance Characteristics

### Compilation
- Proto files downloaded and compiled once
- Generated code cached in `src/generated/`
- Incremental compilation supported

### Runtime
- Zero-cost abstractions with trait dispatch
- Efficient async I/O with Tokio
- Minimal allocations with borrowing

### Binary Size
- Release builds with LTO: ~5-10 MB
- Static linking with musl: ~3-5 MB
- Strip symbols for production: <3 MB

## Idioms and Patterns

### 1. Builder Pattern

Configuration uses the options pattern via `PluginConfig`:

```rust
async fn configure(&self, request: Request<PluginConfig>) -> Result<Response<()>, Status> {
    let config = request.into_inner();
    // Use config.custom_config and config.telemetry
}
```

### 2. Error Handling

Use `?` operator for error propagation:

```rust
async fn handle_request(&self, request: Request<HttpRequest>)
    -> Result<Response<HttpResponse>, Status> {
    let req = request.into_inner();
    // Errors automatically converted via From trait.
    self.validate(&req)?;
    Ok(Response::new(HttpResponse {
        r#continue: true,
        ..Default::default()
    }))
}
```

### 3. Shared State

Use `Arc` and async locks for shared state:

```rust
struct MyPlugin {
    state: Arc<RwLock<HashMap<String, Value>>>,
}

async fn handle_request(&self, request: Request<HttpRequest>)
    -> Result<Response<HttpResponse>, Status> {
    let state = self.state.read().await;
    // Use state
}
```

## Testing Strategy

### Unit Tests
- Test individual components in isolation
- Mock gRPC context
- Use `tokio::test` for async tests

### Integration Tests
- Test full plugin lifecycle
- Use real gRPC connections
- Example plugins serve as integration tests

### Example Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_metadata() {
        let plugin = MyPlugin::new();
        let response = plugin.get_metadata(Request::new(())).await.unwrap();
        assert_eq!(response.into_inner().name, "my-plugin");
    }
}
```

## Backward Compatibility

Following semantic versioning:
- Patch versions: Bug fixes
- Minor versions: New features, backward compatible
- Major versions: Breaking changes

Proto version updates handled via environment variable:
```bash
PROTO_VERSION=v2 cargo build
```

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tonic Documentation](https://docs.rs/tonic/)
- [Tokio Best Practices](https://tokio.rs/tokio/tutorial)
- [mcpd Documentation](https://mozilla-ai.github.io/mcpd/)
