//! # mcpd-plugins-sdk
//!
//! Rust SDK for building [mcpd](https://github.com/mozilla-ai/mcpd) plugins.
//!
//! This SDK provides a simple, trait-based API for creating gRPC plugins that intercept and
//! transform HTTP requests and responses in the mcpd middleware pipeline.
//!
//! ## Features
//!
//! - **Simple trait-based API**: Implement the [`Plugin`] trait with only the methods you need
//! - **Async/await support**: Built on Tokio and Tonic for high-performance async I/O
//! - **Automatic server setup**: [`serve()`] function handles all boilerplate
//! - **Cross-platform**: Unix sockets (Linux/macOS) and TCP support
//! - **Type-safe**: Protocol buffers for serialization
//! - **Graceful shutdown**: SIGINT/SIGTERM handling with cleanup
//!
//! ## Quick Start
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! mcpd-plugins-sdk = "0.1"
//! tokio = { version = "1", features = ["full"] }
//! tonic = "0.12"
//! ```
//!
//! Create a simple plugin:
//!
//! ```rust,no_run
//! use mcpd_plugins_sdk::{
//!     Plugin, serve, Metadata, Capabilities, HttpRequest, HttpResponse,
//!     FLOW_REQUEST,
//! };
//! use tonic::{Request, Response, Status};
//!
//! struct MyPlugin;
//!
//! #[tonic::async_trait]
//! impl Plugin for MyPlugin {
//!     async fn get_metadata(
//!         &self,
//!         _request: Request<()>,
//!     ) -> Result<Response<Metadata>, Status> {
//!         Ok(Response::new(Metadata {
//!             name: "my-plugin".to_string(),
//!             version: "1.0.0".to_string(),
//!             description: "My custom plugin".to_string(),
//!             ..Default::default()
//!         }))
//!     }
//!
//!     async fn get_capabilities(
//!         &self,
//!         _request: Request<()>,
//!     ) -> Result<Response<Capabilities>, Status> {
//!         Ok(Response::new(Capabilities {
//!             flows: vec![FLOW_REQUEST as i32],
//!         }))
//!     }
//!
//!     async fn handle_request(
//!         &self,
//!         request: Request<HttpRequest>,
//!     ) -> Result<Response<HttpResponse>, Status> {
//!         let mut req = request.into_inner();
//!
//!         // Add custom header.
//!         req.headers.insert("X-My-Plugin".to_string(), "processed".to_string());
//!
//!         Ok(Response::new(HttpResponse {
//!             continue_: true,
//!             modified_request: Some(req),
//!             ..Default::default()
//!         }))
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     serve(MyPlugin, None).await?;
//!     Ok(())
//! }
//! ```
//!
//! Run the plugin:
//!
//! ```bash
//! cargo run -- --address /tmp/my-plugin.sock --network unix
//! ```
//!
//! ## Plugin Flows
//!
//! Plugins can participate in two processing flows:
//!
//! - [`FLOW_REQUEST`]: Process incoming HTTP requests before they reach the upstream server
//! - [`FLOW_RESPONSE`]: Process outgoing HTTP responses before they return to the client
//!
//! Declare which flows your plugin supports in the [`Plugin::get_capabilities()`] method.
//!
//! ## Request Handling
//!
//! The [`Plugin::handle_request()`] method receives an [`HttpRequest`] and returns an [`HttpResponse`].
//! You can:
//!
//! 1. **Pass through unchanged**: Return `continue_: true` with no modifications
//! 2. **Transform the request**: Set `modified_request` field with changes
//! 3. **Short-circuit**: Return `continue_: false` with a custom response
//!
//! ## Example: Authentication Plugin
//!
//! ```rust,no_run
//! use mcpd_plugins_sdk::{Plugin, HttpRequest, HttpResponse};
//! use tonic::{Request, Response, Status};
//!
//! struct AuthPlugin;
//!
//! #[tonic::async_trait]
//! impl Plugin for AuthPlugin {
//!     async fn handle_request(
//!         &self,
//!         request: Request<HttpRequest>,
//!     ) -> Result<Response<HttpResponse>, Status> {
//!         let req = request.into_inner();
//!
//!         // Check for Authorization header.
//!         if let Some(auth) = req.headers.get("Authorization") {
//!             if auth.starts_with("Bearer ") {
//!                 // Valid token, continue processing.
//!                 return Ok(Response::new(HttpResponse {
//!                     continue_: true,
//!                     ..Default::default()
//!                 }));
//!             }
//!         }
//!
//!         // No valid token, return 401.
//!         Ok(Response::new(HttpResponse {
//!             continue_: false,
//!             status_code: 401,
//!             body: b"Unauthorized".to_vec(),
//!             ..Default::default()
//!         }))
//!     }
//! }
//! ```

// Generated protobuf code.
#[allow(clippy::all)]
#[allow(missing_docs)]
pub mod proto {
    include!("generated/mozilla.mcpd.plugins.v1.rs");
}

mod constants;
mod error;
mod plugin;
mod server;

// Re-export public API.
pub use constants::{FLOW_REQUEST, FLOW_RESPONSE};
pub use error::{PluginError, Result};
pub use plugin::{Plugin, PluginAdapter};
pub use proto::{
    Capabilities, Flow, HttpRequest, HttpResponse, Metadata, PluginConfig, TelemetryConfig,
};
pub use server::serve;
