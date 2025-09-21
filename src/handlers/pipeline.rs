//! Pipeline management and media analysis HTTP endpoint handlers.
//!
//! This module provides HTTP handlers for managing GStreamer pipeline lifecycles
//! and analyzing remote media files. It supports both custom pipeline creation
//! for advanced users and media analysis for informed processing decisions.
//!
//! # Handler Categories
//!
//! - **Pipeline CRUD**   : Create, read, update, and delete pipeline operations
//! - **Media Analysis**  : Remote media file inspection and metadata extraction
//! - **State Management**: Pipeline status tracking and lifecycle control
//!
//! # Pipeline Lifecycle Management
//!
//! Pipelines are managed through a simple state machine with full CRUD operations:
//! - Creation with validation and unique ID assignment
//! - Status querying for monitoring and debugging
//! - Listing for operational overview
//! - Termination for resource management
//!
//! # Media Analysis Integration
//!
//! The analyze endpoint provides pre-processing inspection of media files,
//! enabling clients to make informed decisions about processing parameters
//! and validate source accessibility before initiating expensive operations.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

// ---

// Import through gateways
use crate::models::{ApiError, CreatePipelineRequest, PipelineInfo, PipelineState};
use crate::services::{get_media_info, validate_pipeline_string};

// ---

// Type alias for shared state
use super::AppState;

/// Creates a new custom GStreamer pipeline from user-provided configuration.
///
/// Accepts a complete GStreamer pipeline string, validates its syntax and structure,
/// then creates a new pipeline entry with a unique identifier. The pipeline is
/// initially in the Created state and ready for execution by external systems.
///
/// # Request Body
/// Expects a JSON payload with pipeline description and GStreamer pipeline string:
/// ```json
/// {
///   "description": "Audio extraction pipeline",
///   "pipeline": "souphttpsrc location=https://example.com/video.mp4 ! decodebin ! audioconvert ! vorbisenc ! oggmux ! filesink location=output.ogg"
/// }
/// ```
///
/// # Validation Process
/// - Ensures pipeline string is not empty or whitespace-only
/// - Verifies proper element connectivity (presence of ! operators)
/// - Uses GStreamer's built-in parser to catch syntax errors
/// - Validates that all referenced elements are available
///
/// # Response Behavior
/// - **200 OK**: Pipeline created successfully with metadata
/// - **400 Bad Request**: Invalid pipeline configuration with detailed error message
///
/// # State Management
/// Created pipelines are stored in application state with:
/// - Unique UUID v4 identifier for tracking
/// - ISO 8601 creation timestamp
/// - Initial state of Created
/// - Complete pipeline string for execution
///
/// # Example Usage
/// ```bash
/// curl -X POST http://localhost:8080/pipelines \
///   -H "Content-Type: application/json" \
///   -d '{"description": "Custom audio extraction", "pipeline": "..."}'
/// ```
pub async fn create_pipeline(
    State(state): State<AppState>,
    Json(payload): Json<CreatePipelineRequest>,
) -> Result<Json<PipelineInfo>, (StatusCode, Json<ApiError>)> {
    // ---

    let pipeline_id = Uuid::new_v4().to_string();

    info!(
        "Creating pipeline: {} - {}",
        pipeline_id, payload.description
    );

    // Validate the pipeline string using our validation service
    if let Err(validation_error) = validate_pipeline_string(&payload.pipeline) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::with_details(
                "Invalid pipeline configuration",
                &validation_error,
            )),
        ));
    }

    let pipeline_info = PipelineInfo {
        id: pipeline_id.clone(),
        description: payload.description,
        state: PipelineState::Created,
        pipeline_string: payload.pipeline,
        created_at: Utc::now().to_rfc3339(),
        source_url: None,
    };

    // Store the pipeline info
    {
        let mut pipelines = state.lock().unwrap();
        pipelines.insert(pipeline_id.clone(), pipeline_info.clone());
    }

    Ok(Json(pipeline_info))
}

/// Lists all currently tracked pipelines with their current states.
///
/// Returns a comprehensive overview of all pipelines in the system, including
/// their current execution states, creation timestamps, and configuration details.
/// This endpoint is useful for operational monitoring and pipeline management.
///
/// # Response Format
/// Returns an array of `PipelineInfo` objects containing complete pipeline metadata:
/// - Unique pipeline identifiers
/// - Human-readable descriptions
/// - Current execution states
/// - Creation timestamps
/// - GStreamer pipeline strings
/// - Source URLs (when applicable)
///
/// # State Information
/// Pipeline states provide insight into execution status:
/// - **Created**: Validated and ready for execution
/// - **Playing**: Currently processing media
/// - **Paused**: Temporarily suspended
/// - **Stopped**: Completed or manually terminated
/// - **Error**: Failed with diagnostic information
///
/// # Operational Use Cases
/// - **Monitoring Dashboards**: Real-time pipeline status overview
/// - **Resource Management**: Identifying active vs completed pipelines
/// - **Debugging**: Historical view of pipeline creation and execution
/// - **Cleanup Operations**: Bulk identification of stopped/error pipelines
///
/// # Example Usage
/// ```bash
/// curl http://localhost:8080/pipelines
/// ```
///
/// # Response Example
/// ```json
/// [
///   {
///     "id": "550e8400-e29b-41d4-a716-446655440000",
///     "description": "Convert to webm",
///     "state": "Created",
///     "pipeline_string": "souphttpsrc location=...",
///     "created_at": "2024-09-21T10:30:00Z",
///     "source_url": "https://example.com/video.mp4"
///   }
/// ]
/// ```
pub async fn list_pipelines(State(state): State<AppState>) -> Json<Vec<PipelineInfo>> {
    // ---

    let pipelines = state.lock().unwrap();
    let pipeline_list: Vec<PipelineInfo> = pipelines.values().cloned().collect();
    Json(pipeline_list)
}

/// Retrieves detailed information about a specific pipeline by ID.
///
/// Returns complete metadata and current state for a single pipeline instance.
/// This endpoint is essential for monitoring individual pipeline progress and
/// debugging specific pipeline issues.
///
/// # Path Parameters
/// - `id`: The unique UUID identifier of the pipeline to retrieve
///
/// # Response Behavior
/// - **200 OK**: Pipeline found and returned with complete metadata
/// - **404 Not Found**: No pipeline exists with the specified ID
///
/// # Use Cases
/// - **Status Monitoring**: Checking individual pipeline execution progress
/// - **Debugging**: Detailed inspection of specific pipeline configuration
/// - **Integration**: Polling for completion status in automated workflows
/// - **Audit Trails**: Historical review of specific pipeline execution
///
/// # Pipeline Information Returned
/// - Complete pipeline configuration and metadata
/// - Current execution state with error details if applicable
/// - Creation timestamp and source URL information
/// - Human-readable description for operational context
///
/// # Example Usage
/// ```bash
/// curl http://localhost:8080/pipelines/550e8400-e29b-41d4-a716-446655440000
/// ```
pub async fn get_pipeline(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PipelineInfo>, (StatusCode, Json<ApiError>)> {
    // ---

    let pipelines = state.lock().unwrap();

    match pipelines.get(&id) {
        Some(pipeline) => Ok(Json(pipeline.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("Pipeline not found")),
        )),
    }
}

/// Stops a running pipeline and updates its state to Stopped.
///
/// Terminates pipeline execution and marks it as stopped in the application state.
/// This operation is useful for resource management, canceling long-running
/// operations, and manual intervention in pipeline execution.
///
/// # Path Parameters
/// - `id`: The unique UUID identifier of the pipeline to stop
///
/// # State Transition
/// The pipeline state is updated to `Stopped` regardless of its previous state.
/// This operation is idempotent - stopping an already stopped pipeline is safe.
///
/// # Response Behavior
/// - **200 OK**: Pipeline successfully stopped with confirmation message
/// - **404 Not Found**: No pipeline exists with the specified ID
///
/// # Resource Management
/// Stopping pipelines is important for:
/// - **Resource Cleanup**: Freeing system resources from abandoned operations
/// - **Error Recovery**: Terminating stuck or misbehaving pipelines
/// - **Operational Control**: Manual intervention in automated workflows
/// - **Testing**: Controlled termination during development and testing
///
/// # Response Format
/// Returns a JSON object with operation confirmation:
/// ```json
/// {
///   "message": "Pipeline stopped successfully",
///   "pipeline_id": "550e8400-e29b-41d4-a716-446655440000"
/// }
/// ```
///
/// # Example Usage
/// ```bash
/// curl -X DELETE http://localhost:8080/pipelines/550e8400-e29b-41d4-a716-446655440000
/// ```
pub async fn stop_pipeline(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    // ---

    let mut pipelines = state.lock().unwrap();

    match pipelines.get_mut(&id) {
        Some(pipeline) => {
            pipeline.state = PipelineState::Stopped;
            info!("Stopped pipeline: {}", id);
            Ok(Json(serde_json::json!({
                "message": "Pipeline stopped successfully",
                "pipeline_id": id
            })))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("Pipeline not found")),
        )),
    }
}

/// Analyzes a remote media file to extract metadata and technical information.
///
/// Performs comprehensive analysis of a media file without downloading or fully
/// processing it. Uses GStreamer's discovery capabilities to extract format
/// information, duration, resolution, and other technical characteristics.
///
/// # Path Parameters
/// - `url`: URL-encoded HTTP(S) URL of the media file to analyze
///
/// # URL Encoding Requirements
/// The media URL must be properly URL-encoded when included in the path:
/// ```bash
/// # Original URL: https://example.com/video.mp4
/// # Encoded URL: https%3A//example.com/video.mp4
/// curl http://localhost:8080/analyze/https%3A//example.com/video.mp4
/// ```
///
/// # Analysis Process
/// - Creates a temporary GStreamer discovery pipeline
/// - Probes media characteristics without full decoding
/// - Extracts technical metadata and format information
/// - Implements timeout protection to prevent hanging
/// - Properly cleans up resources after analysis
///
/// # Response Information
/// Returns comprehensive media metadata including:
/// - **format**: Container format or MIME type
/// - **duration**: Media length in seconds
/// - **width/height**: Video dimensions (when applicable)
/// - **bitrate**: Data rate information (when available)
/// - **analysis_timestamp**: When the analysis was performed
///
/// # Response Behavior
/// - **200 OK**: Analysis completed successfully with media information
/// - **400 Bad Request**: Invalid URL encoding or malformed URL
/// - **422 Unprocessable Entity**: Media file inaccessible or analysis failed
///
/// # Use Cases
/// - **Pre-processing Validation**: Verify media accessibility before expensive operations
/// - **Parameter Optimization**: Choose processing parameters based on source characteristics
/// - **Format Compatibility**: Verify source format compatibility with target operations
/// - **Resource Planning**: Estimate processing requirements based on media characteristics
///
/// # Example Usage
/// ```bash
/// # Analyze Big Buck Bunny sample
/// curl http://localhost:8080/analyze/https%3A//commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4
/// ```
///
/// # Response Example
/// ```json
/// {
///   "url": "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
///   "format": "video/mp4",
///   "duration": 634,
///   "width": 1280,
///   "height": 720,
///   "bitrate": 2000000,
///   "analysis_timestamp": "2024-09-21T10:30:00Z"
/// }
/// ```
pub async fn analyze_media(
    Path(url): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    // ---

    info!("Analyzing media: {}", url);

    // Decode URL parameter
    let decoded_url = urlencoding::decode(&url).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("Invalid URL encoding")),
        )
    })?;

    match get_media_info(&decoded_url) {
        Ok(media_info) => Ok(Json(serde_json::json!({
            "url": decoded_url.as_ref(),
            "format": media_info.format,
            "duration": media_info.duration,
            "width": media_info.width,
            "height": media_info.height,
            "bitrate": media_info.bitrate,
            "analysis_timestamp": Utc::now().to_rfc3339()
        }))),
        Err(e) => {
            warn!("Failed to analyze media {}: {}", decoded_url, e);
            Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiError::with_details(
                    "Failed to analyze media",
                    &format!("Could not retrieve media information: {e}"),
                )),
            ))
        }
    }
}
