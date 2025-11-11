//! Authentication plugin that validates Bearer tokens.
//!
//! This plugin demonstrates how to implement request validation and
//! short-circuit the processing pipeline by returning early responses.

use mcpd_plugins_sdk::{
    serve, Capabilities, HttpRequest, HttpResponse, Metadata, Plugin, PluginConfig, FLOW_REQUEST,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

struct AuthPlugin {
    valid_tokens: Arc<RwLock<HashSet<String>>>,
}

impl AuthPlugin {
    fn new() -> Self {
        let mut tokens = HashSet::new();
        // Add some default tokens for demo purposes.
        tokens.insert("demo-token-123".to_string());
        tokens.insert("test-token-456".to_string());

        Self {
            valid_tokens: Arc::new(RwLock::new(tokens)),
        }
    }
}

#[tonic::async_trait]
impl Plugin for AuthPlugin {
    async fn get_metadata(&self, _request: Request<()>) -> Result<Response<Metadata>, Status> {
        Ok(Response::new(Metadata {
            name: "auth-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Bearer token authentication plugin".to_string(),
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

    async fn configure(&self, request: Request<PluginConfig>) -> Result<Response<()>, Status> {
        let config = request.into_inner();

        tracing::info!("Configuring auth plugin with custom config");

        // Parse configuration.
        if let Some(tokens_str) = config.custom_config.get("valid_tokens") {
            let tokens: Vec<String> = tokens_str.split(',').map(|s| s.to_string()).collect();
            let mut valid_tokens = self.valid_tokens.write().await;
            valid_tokens.clear();
            valid_tokens.extend(tokens);
            tracing::info!("Loaded {} valid tokens from config", valid_tokens.len());
        }

        Ok(Response::new(()))
    }

    async fn handle_request(
        &self,
        request: Request<HttpRequest>,
    ) -> Result<Response<HttpResponse>, Status> {
        let req = request.into_inner();

        tracing::info!("Authenticating {} request to {}", req.method, req.path);

        // Skip health check endpoints.
        if req.path == "/health" || req.path == "/ready" {
            return Ok(Response::new(HttpResponse {
                r#continue: true,
                ..Default::default()
            }));
        }

        // Check for Authorization header.
        if let Some(auth_header) = req.headers.get("Authorization") {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                // Validate token.
                let valid_tokens = self.valid_tokens.read().await;
                if valid_tokens.contains(token) {
                    tracing::info!("Valid token provided, allowing request");
                    return Ok(Response::new(HttpResponse {
                        r#continue: true,
                        ..Default::default()
                    }));
                } else {
                    tracing::warn!("Invalid token provided");
                }
            } else {
                tracing::warn!("Authorization header present but not Bearer token");
            }
        } else {
            tracing::warn!("No Authorization header provided");
        }

        // Return 401 Unauthorized.
        let body = serde_json::json!({
            "error": "unauthorized",
            "message": "Valid Bearer token required"
        });

        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert(
            "WWW-Authenticate".to_string(),
            "Bearer realm=\"mcpd\"".to_string(),
        );

        Ok(Response::new(HttpResponse {
            r#continue: false,
            status_code: 401,
            headers,
            body: body.to_string().into_bytes(),
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

    tracing::info!("Starting auth plugin");

    // Serve the plugin.
    serve(AuthPlugin::new(), None).await?;

    Ok(())
}
