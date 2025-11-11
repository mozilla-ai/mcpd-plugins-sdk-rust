use crate::proto::{
    plugin_server::Plugin as PluginService, Capabilities, HttpRequest, HttpResponse, Metadata,
    PluginConfig,
};
use tonic::{Request, Response, Status};

/// Main Plugin trait that all plugins must implement.
///
/// This trait provides default implementations for all methods, allowing plugins
/// to override only the methods they need. By default:
/// - Health and readiness checks return Ok
/// - Configure and Stop do nothing
/// - GetMetadata returns empty metadata (should be overridden)
/// - GetCapabilities returns no flows (should be overridden)
/// - HandleRequest and HandleResponse pass through requests unchanged
///
/// # Example
///
/// ```rust,no_run
/// use mcpd_plugins_sdk::{Plugin, Metadata, Capabilities, FLOW_REQUEST};
/// use tonic::{Request, Response, Status};
///
/// struct MyPlugin;
///
/// #[tonic::async_trait]
/// impl Plugin for MyPlugin {
///     async fn get_metadata(
///         &self,
///         _request: Request<()>,
///     ) -> Result<Response<Metadata>, Status> {
///         Ok(Response::new(Metadata {
///             name: "my-plugin".to_string(),
///             version: "1.0.0".to_string(),
///             description: "My custom plugin".to_string(),
///             ..Default::default()
///         }))
///     }
///
///     async fn get_capabilities(
///         &self,
///         _request: Request<()>,
///     ) -> Result<Response<Capabilities>, Status> {
///         Ok(Response::new(Capabilities {
///             flows: vec![FLOW_REQUEST as i32],
///         }))
///     }
/// }
/// ```
#[tonic::async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Returns plugin metadata (name, version, description, etc.).
    ///
    /// This method should be overridden to provide plugin identification.
    async fn get_metadata(&self, _request: Request<()>) -> Result<Response<Metadata>, Status> {
        Ok(Response::new(Metadata::default()))
    }

    /// Returns the capabilities of this plugin (which flows it supports).
    ///
    /// This method should be overridden to declare which processing flows
    /// (REQUEST, RESPONSE, or both) the plugin participates in.
    async fn get_capabilities(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Capabilities>, Status> {
        Ok(Response::new(Capabilities { flows: vec![] }))
    }

    /// Configures the plugin with host-provided settings.
    ///
    /// Called once during plugin initialization. Override this to parse
    /// custom configuration and initialize resources.
    async fn configure(&self, _request: Request<PluginConfig>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    /// Stops the plugin and cleans up resources.
    ///
    /// Called during graceful shutdown. Override this to close connections,
    /// flush buffers, and release resources.
    async fn stop(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    /// Health check endpoint.
    ///
    /// Returns Ok if the plugin is alive and operational.
    async fn check_health(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    /// Readiness check endpoint.
    ///
    /// Returns Ok if the plugin is ready to handle requests.
    async fn check_ready(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    /// Handles incoming HTTP requests.
    ///
    /// Override this method to process, transform, or reject HTTP requests.
    /// Return `HttpResponse { continue_: true, .. }` to pass the request to the next handler.
    /// Return `HttpResponse { continue_: false, status_code, .. }` to short-circuit and
    /// return a response directly to the client.
    async fn handle_request(
        &self,
        request: Request<HttpRequest>,
    ) -> Result<Response<HttpResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(HttpResponse {
            r#continue: true,
            ..Default::default()
        }))
    }

    /// Handles outgoing HTTP responses.
    ///
    /// Override this method to process, transform, or modify HTTP responses
    /// before they are returned to the client.
    async fn handle_response(
        &self,
        response: Request<HttpResponse>,
    ) -> Result<Response<HttpResponse>, Status> {
        let resp = response.into_inner();
        Ok(Response::new(HttpResponse {
            r#continue: true,
            status_code: resp.status_code,
            headers: resp.headers,
            body: resp.body,
            ..Default::default()
        }))
    }
}

/// Adapter that implements the generated gRPC service trait using our Plugin trait.
///
/// This bridges between the tonic-generated PluginService trait and our custom Plugin trait.
pub struct PluginAdapter<P: Plugin> {
    plugin: P,
}

impl<P: Plugin> PluginAdapter<P> {
    pub fn new(plugin: P) -> Self {
        Self { plugin }
    }
}

#[tonic::async_trait]
impl<P: Plugin> PluginService for PluginAdapter<P> {
    async fn get_metadata(&self, request: Request<()>) -> Result<Response<Metadata>, Status> {
        self.plugin.get_metadata(request).await
    }

    async fn get_capabilities(
        &self,
        request: Request<()>,
    ) -> Result<Response<Capabilities>, Status> {
        self.plugin.get_capabilities(request).await
    }

    async fn configure(&self, request: Request<PluginConfig>) -> Result<Response<()>, Status> {
        self.plugin.configure(request).await
    }

    async fn stop(&self, request: Request<()>) -> Result<Response<()>, Status> {
        self.plugin.stop(request).await
    }

    async fn check_health(&self, request: Request<()>) -> Result<Response<()>, Status> {
        self.plugin.check_health(request).await
    }

    async fn check_ready(&self, request: Request<()>) -> Result<Response<()>, Status> {
        self.plugin.check_ready(request).await
    }

    async fn handle_request(
        &self,
        request: Request<HttpRequest>,
    ) -> Result<Response<HttpResponse>, Status> {
        self.plugin.handle_request(request).await
    }

    async fn handle_response(
        &self,
        response: Request<HttpResponse>,
    ) -> Result<Response<HttpResponse>, Status> {
        self.plugin.handle_response(response).await
    }
}
