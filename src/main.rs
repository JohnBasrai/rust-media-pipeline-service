// EMBP Gateway: main.rs controls module boundaries and public exports
mod handlers;
mod models;
mod services;

use axum::{
    routing::{delete, get, post},
    Router,
};
use clap::Parser;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

// Import through module gateways
use handlers::{
    // ---
    analyze_media,
    convert_media,
    create_pipeline,
    create_stream,
    generate_thumbnail,
    get_pipeline,
    health_check,
    list_pipelines,
    list_sample_media,
    stop_pipeline,
    AppState,
};

// Public exports (if this were a library crate)
pub use models::{
    // ---
    ConvertRequest,
    ConvertResponse,
    PipelineInfo,
    PipelineState,
    StreamRequest,
    StreamResponse,
    ThumbnailRequest,
    ThumbnailResponse,
};
pub use services::PipelineService;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    //
    /// Port to bind the server to
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Host address to bind the server to  
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ---

    // Parse command line arguments
    let cli = Cli::parse();

    // ---

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize GStreamer
    gstreamer::init()?;
    info!("GStreamer initialized successfully!");
    info!("Version: {}", gstreamer::version_string());

    // ---

    // Create shared application state
    let app_state: AppState = Arc::new(Mutex::new(HashMap::new()));

    // Build our application with routes
    let app = Router::new()
        .route("/", get(health_check))
        .route("/analyze/*url", get(analyze_media))
        .route("/convert", post(convert_media))
        .route("/health", get(health_check))
        .route("/pipelines", get(list_pipelines))
        .route("/pipelines", post(create_pipeline))
        .route("/pipelines/:id", delete(stop_pipeline))
        .route("/pipelines/:id", get(get_pipeline))
        .route("/samples", get(list_sample_media))
        .route("/stream", post(create_stream))
        .route("/thumbnail", post(generate_thumbnail))
        .with_state(app_state);

    // ---

    // Create bind address from CLI args
    let bind_addr = format!("{}:{}", cli.host, cli.port);

    // Start the server with graceful shutdown handling
    let listener = TcpListener::bind(&bind_addr).await?;
    info!("Media Pipeline Service running on http://{}", bind_addr);
    info!("Try: curl http://localhost:{}/samples", cli.port);
    info!("Or:  curl http://localhost:{}/analyze/https%3A//commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4", cli.port);

    // TODO: Add comprehensive signal handling for production:
    // - Stop active GStreamer pipelines before exit
    // - Implement graceful shutdown timeout
    // - Handle additional signals (SIGTERM) for containerized environments

    let result = tokio::select! {
        result = axum::serve(listener, app) => {
            result
        }
        _ = signal::ctrl_c() => {
            info!("Shutdown signal received, stopping server...");
            Ok(())
        }
    };

    if let Err(err) = &result {
        tracing::error!("Server error: {}", err);
    }

    result.map_err(anyhow::Error::from)
}
