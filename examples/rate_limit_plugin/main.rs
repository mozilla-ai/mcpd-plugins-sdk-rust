//! Rate limiting plugin using token bucket algorithm.
//!
//! This plugin demonstrates stateful request processing with per-client
//! rate limiting and configuration via the Configure method.

use mcpd_plugins_sdk::{
    serve, Capabilities, HttpRequest, HttpResponse, Metadata, Plugin, PluginConfig, FLOW_REQUEST,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }

    fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn available_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }
}

struct RateLimitPlugin {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    max_requests: f64,
    window_duration: Duration,
}

impl RateLimitPlugin {
    fn new() -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            max_requests: 10.0,
            window_duration: Duration::from_secs(60),
        }
    }
}

#[tonic::async_trait]
impl Plugin for RateLimitPlugin {
    async fn get_metadata(&self, _request: Request<()>) -> Result<Response<Metadata>, Status> {
        Ok(Response::new(Metadata {
            name: "rate-limit-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Token bucket rate limiting plugin".to_string(),
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

        tracing::info!("Configuring rate limit plugin");

        // Parse max_requests from config.
        if let Some(max_req_str) = config.custom_config.get("max_requests") {
            if let Ok(max_req) = max_req_str.parse::<f64>() {
                tracing::info!("Setting max_requests to {}", max_req);
            }
        }

        // Parse window_duration from config.
        if let Some(window_str) = config.custom_config.get("window_seconds") {
            if let Ok(window_secs) = window_str.parse::<u64>() {
                tracing::info!("Setting window duration to {} seconds", window_secs);
            }
        }

        Ok(Response::new(()))
    }

    async fn handle_request(
        &self,
        request: Request<HttpRequest>,
    ) -> Result<Response<HttpResponse>, Status> {
        let req = request.into_inner();

        tracing::debug!("Rate limiting {} request to {}", req.method, req.path);

        // Use remote_addr as the client identifier.
        let client_id = req.remote_addr.clone();

        let mut buckets = self.buckets.lock().await;

        // Get or create bucket for this client.
        let bucket = buckets.entry(client_id.clone()).or_insert_with(|| {
            let refill_rate = self.max_requests / self.window_duration.as_secs_f64();
            TokenBucket::new(self.max_requests, refill_rate)
        });

        // Try to consume a token.
        if bucket.try_consume() {
            let available = bucket.available_tokens();
            tracing::debug!(
                "Request allowed for client {} ({:.1} tokens remaining)",
                client_id,
                available
            );

            // Add rate limit headers.
            let mut headers = std::collections::HashMap::new();
            headers.insert(
                "X-RateLimit-Limit".to_string(),
                self.max_requests.to_string(),
            );
            headers.insert(
                "X-RateLimit-Remaining".to_string(),
                available.floor().to_string(),
            );

            Ok(Response::new(HttpResponse {
                r#continue: true,
                headers,
                ..Default::default()
            }))
        } else {
            tracing::warn!("Rate limit exceeded for client {}", client_id);

            // Return 429 Too Many Requests.
            let body = serde_json::json!({
                "error": "rate_limit_exceeded",
                "message": "Too many requests, please try again later"
            });

            let mut headers = std::collections::HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            headers.insert(
                "X-RateLimit-Limit".to_string(),
                self.max_requests.to_string(),
            );
            headers.insert("X-RateLimit-Remaining".to_string(), "0".to_string());
            headers.insert(
                "Retry-After".to_string(),
                self.window_duration.as_secs().to_string(),
            );

            Ok(Response::new(HttpResponse {
                r#continue: false,
                status_code: 429,
                headers,
                body: body.to_string().into_bytes(),
                ..Default::default()
            }))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing.
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    tracing::info!("Starting rate limit plugin");

    // Serve the plugin.
    serve(RateLimitPlugin::new(), None).await?;

    Ok(())
}
