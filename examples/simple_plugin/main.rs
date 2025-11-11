//! Simple plugin that adds a custom header to all requests.
//!
//! This demonstrates the minimal implementation of a plugin that processes
//! HTTP requests and adds custom metadata.

use mcpd_plugins_sdk::{
    serve, Capabilities, HttpRequest, HttpResponse, Metadata, Plugin, FLOW_REQUEST,
};
use tonic::{Request, Response, Status};
use tracing_subscriber;

struct SimplePlugin;

#[tonic::async_trait]
impl Plugin for SimplePlugin {
    async fn get_metadata(&self, _request: Request<()>) -> Result<Response<Metadata>, Status> {
        Ok(Response::new(Metadata {
            name: "simple-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A simple plugin that adds custom headers".to_string(),
            commit_hash: env!("CARGO_PKG_VERSION").to_string(),
            build_date: env!("CARGO_PKG_VERSION").to_string(),
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

        // Log the request.
        tracing::info!("Processing {} request to {}", req.method, req.path);

        // Add custom header.
        req.headers
            .insert("X-Simple-Plugin".to_string(), "processed".to_string());
        req.headers.insert(
            "X-Plugin-Version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );

        // Return modified request.
        Ok(Response::new(HttpResponse {
            r#continue: true,
            modified_request: Some(req),
            ..Default::default()
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing.
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    tracing::info!("Starting simple plugin");

    // Serve the plugin.
    serve(SimplePlugin, None).await?;

    Ok(())
}
