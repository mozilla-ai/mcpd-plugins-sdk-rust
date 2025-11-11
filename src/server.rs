use crate::plugin::{Plugin, PluginAdapter};
use crate::proto::plugin_server::PluginServer;
use crate::{PluginError, Result};
use clap::Parser;
use std::path::PathBuf;
use tokio::signal;
use tonic::transport::Server;
use tracing::{info, warn};

#[cfg(unix)]
use tokio::net::UnixListener;

/// Command-line arguments for the plugin server.
#[derive(Parser, Debug)]
#[command(author, version, about = "mcpd plugin server", long_about = None)]
struct Args {
    /// Address to bind to (socket path for unix, host:port for tcp).
    #[arg(long)]
    address: String,

    /// Network type (unix or tcp).
    #[arg(long, default_value = "unix")]
    network: String,
}

/// Serves a plugin on the specified address.
///
/// This is the main entry point for running a plugin. It handles:
/// - Command-line argument parsing
/// - Server setup (Unix socket or TCP)
/// - Graceful shutdown on SIGINT/SIGTERM
/// - Automatic cleanup of Unix socket files
///
/// # Arguments
///
/// * `plugin` - The plugin implementation to serve
/// * `args` - Optional command-line arguments (defaults to std::env::args())
///
/// # Example
///
/// ```rust,no_run
/// use mcpd_plugins_sdk::{Plugin, serve};
///
/// struct MyPlugin;
///
/// #[tonic::async_trait]
/// impl Plugin for MyPlugin {
///     // Implementation...
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     serve(MyPlugin, None).await?;
///     Ok(())
/// }
/// ```
pub async fn serve<P: Plugin>(plugin: P, args: Option<Vec<String>>) -> Result<()> {
    // Parse command-line arguments.
    let args = if let Some(args) = args {
        Args::parse_from(args)
    } else {
        Args::parse()
    };

    info!(
        "Starting plugin server on {} ({})",
        args.address, args.network
    );

    // Create the plugin adapter.
    let adapter = PluginAdapter::new(plugin);
    let service = PluginServer::new(adapter);

    // Serve based on network type.
    match args.network.as_str() {
        "unix" => serve_unix(service, &args.address).await,
        "tcp" => serve_tcp(service, &args.address).await,
        network => Err(PluginError::Configuration(format!(
            "Unsupported network type: {}",
            network
        ))),
    }
}

#[cfg(unix)]
async fn serve_unix<S>(service: S, address: &str) -> Result<()>
where
    S: tonic::codegen::Service<
            http::Request<tonic::body::BoxBody>,
            Response = http::Response<tonic::body::BoxBody>,
            Error = std::convert::Infallible,
        > + tonic::server::NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    use tokio_stream::wrappers::UnixListenerStream;

    let path = PathBuf::from(address);

    // Remove existing socket file if it exists.
    if path.exists() {
        warn!("Removing existing socket file: {}", address);
        std::fs::remove_file(&path)?;
    }

    // Create Unix listener.
    let listener = UnixListener::bind(&path)?;
    let stream = UnixListenerStream::new(listener);

    info!("Listening on Unix socket: {}", address);

    // Serve with graceful shutdown.
    Server::builder()
        .add_service(service)
        .serve_with_incoming_shutdown(stream, shutdown_signal())
        .await?;

    // Clean up socket file on shutdown.
    if path.exists() {
        info!("Cleaning up socket file: {}", address);
        let _ = std::fs::remove_file(&path);
    }

    Ok(())
}

#[cfg(not(unix))]
async fn serve_unix<S>(_service: S, _address: &str) -> Result<()>
where
    S: tonic::codegen::Service<
            http::Request<tonic::body::BoxBody>,
            Response = http::Response<tonic::body::BoxBody>,
            Error = std::convert::Infallible,
        > + tonic::server::NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    Err(PluginError::Configuration(
        "Unix sockets not supported on this platform".to_string(),
    ))
}

async fn serve_tcp<S>(service: S, address: &str) -> Result<()>
where
    S: tonic::codegen::Service<
            http::Request<tonic::body::BoxBody>,
            Response = http::Response<tonic::body::BoxBody>,
            Error = std::convert::Infallible,
        > + tonic::server::NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    let addr = address
        .parse()
        .map_err(|e| PluginError::Configuration(format!("Invalid TCP address: {}", e)))?;

    info!("Listening on TCP: {}", address);

    // Serve with graceful shutdown.
    Server::builder()
        .add_service(service)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

/// Waits for a shutdown signal (SIGINT or SIGTERM).
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received SIGINT, shutting down gracefully");
        }
        _ = terminate => {
            info!("Received SIGTERM, shutting down gracefully");
        }
    }
}
