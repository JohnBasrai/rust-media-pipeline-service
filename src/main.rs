//! Media Pipeline Service - A REST API for media processing using GStreamer and Rust.
//!
//! This service provides a comprehensive REST API for media processing operations
//! including format conversion, thumbnail generation, and adaptive streaming.
//! Built with the Axum web framework and GStreamer multimedia framework, it
//! demonstrates modern Rust architecture patterns and professional development practices.
//!
//! # Core Features
//!
//! - **Format Conversion**   : Transform media between WebM, MP4, and AVI formats
//! - **Thumbnail Generation**: Extract still images from video content at specified timestamps
//! - **Adaptive Streaming**  : Create HLS streams for web and mobile delivery
//! - **Custom Pipelines**    : Support for user-defined GStreamer pipeline execution
//! - **Media Analysis**      : Extract metadata and technical information from remote media files
//! - **Pipeline Management** : Full CRUD operations for pipeline lifecycle management
//!
//! # Architecture
//!
//! The service implements the **Explicit Module Boundary Pattern (EMBP)** for clean
//! code organization with three main layers:
//!
//! - **Handlers**: HTTP endpoint logic with request/response handling
//! - **Services**: Business logic and GStreamer integration
//! - **Models**  : Data transfer objects and state management
//!
//! Each layer maintains clear boundaries through gateway modules that control
//! public API exposure and enable maintainable refactoring.
//!
//! # API Endpoints
//!
//! ## Media Processing
//! - `POST /convert`      - Convert media between formats
//! - `POST /thumbnail`    - Generate thumbnails from video content
//! - `POST /stream`       - Create adaptive streaming pipelines
//! - `GET /analyze/{url}` - Analyze remote media file metadata
//!
//! ## Pipeline Management
//! - `GET /pipelines`         - List all active pipelines
//! - `POST /pipelines`        - Create custom GStreamer pipelines
//! - `GET /pipelines/{id}`    - Get specific pipeline status
//! - `DELETE /pipelines/{id}` - Stop pipeline execution
//!
//! ## Service Operations
//! - `GET /health`  - Service health check and capability reporting
//! - `GET /samples` - List curated sample media for testing
//!
//! # Documentation and Examples
//!
//! For installation instructions, usage examples, and detailed API documentation,
//! see the project README and run `cargo doc --open` to browse the complete
//! generated documentation.
//!
//! # Technology Stack
//!
//! - **Axum**     : Modern async web framework for HTTP API
//! - **GStreamer**: Multimedia framework for pipeline-based media processing
//! - **Tokio**    : Async runtime for concurrent request handling
//! - **Serde**    : JSON serialization for API requests and responses
//! - **Tracing**  : Structured logging and observability
//! - **Clap**     : Command-line argument parsing with derive macros

// ---

// EMBP Gateway: main.rs controls module boundaries and public exports
mod handlers;
mod models;
mod services;

use axum::{
    routing::{delete, get, post},
    Router,
};
use clap::{Parser, ValueEnum};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

// ---

// Import through module gateways
use handlers::{
    analyze_media, convert_media, create_pipeline, create_stream, generate_thumbnail, get_pipeline,
    health_check, list_pipelines, list_sample_media, stop_pipeline, AppState,
};

/// Color output control for terminal compatibility.
///
/// Provides fine-grained control over colored log output to ensure compatibility
/// with different terminal environments and output redirection scenarios.
#[derive(Clone, Debug, ValueEnum)]
enum ColorWhen {
    /// Automatically detect terminal capabilities (default)
    Auto,
    /// Force colored output regardless of terminal detection
    Always,
    /// Disable colored output completely
    Never,
}

/// Command-line interface configuration for the Media Pipeline Service.
///
/// Provides comprehensive control over service binding, logging, and operational
/// parameters. Supports both development and production deployment scenarios
/// with sensible defaults and flexible overrides.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // ---
    /// Port to bind the server to
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Host address to bind the server to  
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Control colored log output for terminal compatibility
    #[arg(long, value_enum, default_value_t = ColorWhen::Auto)]
    color: ColorWhen,
}

/// Application entry point and service initialization.
///
/// Handles command-line argument parsing, GStreamer initialization, HTTP routing
/// setup, and graceful shutdown coordination. Implements comprehensive error
/// handling and logging for operational visibility.
///
/// # Initialization Sequence
/// 1. Parse command-line arguments for service configuration
/// 2. Initialize structured logging with terminal-aware colorization
/// 3. Initialize GStreamer multimedia framework
/// 4. Create shared application state for pipeline tracking
/// 5. Configure HTTP routing with all API endpoints
/// 6. Start HTTP server with graceful shutdown handling
///
/// # Error Handling
/// All initialization errors are propagated using `anyhow::Error` for
/// comprehensive error context and debugging information.
///
/// # Graceful Shutdown
/// The service responds to SIGINT (Ctrl+C) signals by cleanly shutting down
/// the HTTP server and releasing resources. Future enhancements will include
/// active pipeline termination and extended signal handling.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ---

    // Parse command line arguments
    let cli = Cli::parse();

    // ---

    // Initialize tracing with smart colorization
    let use_color = match cli.color {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => {
            // Check if stdout is a terminal and not being redirected
            std::io::IsTerminal::is_terminal(&std::io::stdout())
        }
    };

    tracing_subscriber::fmt().with_ansi(use_color).init();

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
