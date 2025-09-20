// EMBP Gateway: main.rs controls module boundaries and public exports
mod handlers;
mod models;
mod services;

// Internal imports for main() function
use axum::{
    routing::{delete, get, post},
    Router,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tracing::info;

// Import through module gateways
use handlers::{
    convert_media, create_pipeline, create_stream, generate_thumbnail, get_pipeline, health_check,
    list_pipelines, list_sample_media, stop_pipeline,
};

// Public exports (if this were a library crate)
pub use models::{
    ConvertRequest, ConvertResponse, PipelineInfo, PipelineState, StreamRequest, StreamResponse,
    ThumbnailRequest, ThumbnailResponse,
};
pub use services::PipelineService;

// Global state for managing pipelines
type AppState = Arc<Mutex<HashMap<String, PipelineInfo>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize GStreamer
    gstreamer::init()?;
    info!("🎬 GStreamer initialized successfully!");
    info!("📡 Version: {}", gstreamer::version_string());

    // Create shared application state
    let app_state: AppState = Arc::new(Mutex::new(HashMap::new()));

    // Build our application with routes
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/pipelines", post(create_pipeline))
        .route("/pipelines", get(list_pipelines))
        .route("/pipelines/:id", get(get_pipeline))
        .route("/pipelines/:id", delete(stop_pipeline))
        .route("/convert", post(convert_media))
        .route("/thumbnail", post(generate_thumbnail))
        .route("/stream", post(create_stream))
        .route("/samples", get(list_sample_media))
        .with_state(app_state);

    // Start the server
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!("🚀 Media Pipeline Service running on http://0.0.0.0:8080");
    info!("📚 Try: curl http://localhost:8080/samples");

    axum::serve(listener, app).await?;

    Ok(())
}
