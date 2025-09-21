//! Media processing HTTP endpoint handlers for format conversion, thumbnails, and streaming.
//!
//! This module provides HTTP handlers for the core media processing operations
//! supported by the service. It handles format conversion between common video
//! formats, thumbnail extraction from video sources, and streaming pipeline
//! creation for adaptive media delivery.
//!
//! # Handler Categories
//!
//! - **Format Conversion**   : Transform media between WebM, MP4, and AVI formats
//! - **Thumbnail Generation**: Extract still images from video content at specified timestamps
//! - **Streaming Pipeline**  : Create HLS streams for adaptive media delivery
//!
//! # Processing Architecture
//!
//! All handlers follow a consistent pattern:
//! 1. URL validation and media analysis
//! 2. Pipeline string generation using optimized templates
//! 3. Pipeline validation before storage
//! 4. State management with unique tracking IDs
//! 5. Asynchronous execution with status tracking
//!
//! # Error Handling Strategy
//!
//! Handlers provide comprehensive error reporting with HTTP status codes
//! that distinguish between client errors (validation failures) and server
//! errors (processing issues), enabling appropriate client retry logic.

use axum::{extract::State, http::StatusCode, response::Json};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};
use uuid::Uuid;

// ---

// Import through gateways
use crate::models::{
    ApiError, ConvertRequest, ConvertResponse, PipelineInfo, PipelineState, StreamRequest,
    StreamResponse, ThumbnailInfo, ThumbnailRequest, ThumbnailResponse,
};
use crate::services::{
    create_conversion_pipeline, create_hls_stream_pipeline, create_thumbnail_pipeline,
    get_media_info, validate_pipeline_string,
};

// ---

// Type alias for shared state
pub type AppState = Arc<Mutex<HashMap<String, PipelineInfo>>>;

/// Initiates media format conversion between supported video formats.
///
/// Creates a conversion pipeline that transforms the source media into the specified
/// output format using optimized GStreamer pipelines. Supports conversion between
/// WebM, MP4, and AVI formats with appropriate codec selection for each target.
///
/// # Request Body
/// Expects a JSON payload specifying source URL and target format:
/// ```json
/// {
///   "source_url": "https://example.com/video.mp4",
///   "output_format": "webm"
/// }
/// ```
///
/// # Supported Format Conversions
/// - **webm**: VP8 video codec with WebM container (web-optimized, open source)
/// - **mp4**: H.264 video codec with MP4 container (broad compatibility)
/// - **avi**: H.264 video codec with AVI container (legacy compatibility)
///
/// # Validation Process
/// 1. **URL Validation**: Ensures source URL uses HTTP(S) protocol
/// 2. **Media Analysis**: Attempts to probe source media characteristics
/// 3. **Pipeline Generation**: Creates optimized conversion pipeline
/// 4. **Pipeline Validation**: Verifies generated pipeline syntax
/// 5. **State Storage**: Records pipeline info for tracking
///
/// # Response Behavior
/// - **200 OK**: Conversion pipeline created successfully
/// - **400 Bad Request**: Invalid source URL or unsupported format
/// - **500 Internal Server Error**: Pipeline generation or validation failure
///
/// # Processing Characteristics
/// - **Asynchronous**: Conversion runs independently of HTTP request
/// - **Tracked**: Pipeline ID enables status monitoring
/// - **Optimized**: Format-specific codec and container selection
/// - **Validated**: Source media accessibility verified when possible
///
/// # Example Usage
/// ```bash
/// curl -X POST http://localhost:8080/convert \
///   -H "Content-Type: application/json" \
///   -d '{
///     "source_url": "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
///     "output_format": "webm"
///   }'
/// ```
///
/// # Response Example
/// ```json
/// {
///   "pipeline_id": "550e8400-e29b-41d4-a716-446655440000",
///   "status": "created",
///   "message": "Conversion to webm initiated",
///   "estimated_duration": "2-5 minutes"
/// }
/// ```
pub async fn convert_media(
    State(state): State<AppState>,
    Json(payload): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, (StatusCode, Json<ApiError>)> {
    // ---

    let pipeline_id = Uuid::new_v4().to_string();

    info!(
        "Converting media: {} -> {}",
        payload.source_url, payload.output_format
    );

    // Validate URL format
    if !payload.source_url.starts_with("http") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("Source URL must be a valid HTTP(S) URL")),
        ));
    }

    // Try to get media info first to validate the source
    match get_media_info(&payload.source_url) {
        Ok(media_info) => {
            info!("Source media format: {}", media_info.format);
        }
        Err(e) => {
            warn!("Could not analyze source media: {}", e);
            // Continue anyway - the source might still be valid for streaming
        }
    }

    // Create output path
    let output_path = format!("output_{}.{}", pipeline_id, payload.output_format);

    // Use validation service to create proper pipeline
    let pipeline_string =
        match create_conversion_pipeline(&payload.source_url, &payload.output_format, &output_path)
        {
            Ok(pipeline) => pipeline,
            Err(e) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError::with_details("Unsupported format conversion", &e)),
                ));
            }
        };

    // Validate the generated pipeline
    if let Err(validation_error) = validate_pipeline_string(&pipeline_string) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Generated invalid pipeline",
                &validation_error,
            )),
        ));
    }

    // Store pipeline info
    let pipeline_info = PipelineInfo {
        id: pipeline_id.clone(),
        description: format!("Convert to {}", payload.output_format),
        state: PipelineState::Created,
        pipeline_string,
        created_at: Utc::now().to_rfc3339(),
        source_url: Some(payload.source_url),
    };

    {
        let mut pipelines = state.lock().unwrap();
        pipelines.insert(pipeline_id.clone(), pipeline_info);
    }

    Ok(Json(ConvertResponse {
        pipeline_id,
        status: "created".to_string(),
        message: format!("Conversion to {} initiated", payload.output_format),
        estimated_duration: Some("2-5 minutes".to_string()),
    }))
}

/// Generates a thumbnail image from a video source at a specified timestamp.
///
/// Extracts a single frame from the video at the requested timestamp and converts
/// it to a PNG image with optional resizing. Provides precise control over output
/// dimensions and extraction timing for various use cases.
///
/// # Request Body
/// Expects a JSON payload with source URL and optional parameters:
/// ```json
/// {
///   "source_url": "https://example.com/video.mp4",
///   "timestamp": "00:01:30",
///   "width": 640,
///   "height": 480
/// }
/// ```
///
/// # Parameters
/// - **source_url**: HTTP(S) URL of the source video (required)
/// - **timestamp**: Time position in HH:MM:SS format (optional, defaults to "00:00:10")
/// - **width**: Output width in pixels (optional, defaults to 320)
/// - **height**: Output height in pixels (optional, defaults to 240)
///
/// # Thumbnail Characteristics
/// - **Format**: PNG for lossless quality and transparency support
/// - **Scaling**: Images are scaled to exact dimensions (aspect ratio not preserved)
/// - **Quality**: Full color depth with no compression artifacts
/// - **Positioning**: Extracted from specified timestamp position
///
/// # Validation and Processing
/// 1. **URL Protocol Validation**: Ensures HTTP(S) source URLs
/// 2. **Media Type Verification**: Attempts to confirm video content
/// 3. **Pipeline Generation**: Creates optimized thumbnail extraction pipeline
/// 4. **Dimension Validation**: Applies default values for missing parameters
/// 5. **State Tracking**: Records pipeline for monitoring and management
///
/// # Response Behavior
/// - **200 OK**: Thumbnail generation pipeline created successfully
/// - **400 Bad Request**: Invalid source URL or parameters
/// - **500 Internal Server Error**: Pipeline generation failure
///
/// # Use Cases
/// - **Video Previews**: Generate preview images for video catalogs
/// - **Content Moderation**: Extract frames for automated content analysis
/// - **Poster Generation**: Create representative images for video content
/// - **Quality Assessment**: Visual inspection of video content at specific moments
///
/// # Example Usage
/// ```bash
/// curl -X POST http://localhost:8080/thumbnail \
///   -H "Content-Type: application/json" \
///   -d '{
///     "source_url": "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
///     "timestamp": "00:01:30",
///     "width": 640,
///     "height": 480
///   }'
/// ```
///
/// # Response Example
/// ```json
/// {
///   "pipeline_id": "550e8400-e29b-41d4-a716-446655440001",
///   "status": "created",
///   "message": "Thumbnail generation initiated",
///   "output_info": {
///     "width": 640,
///     "height": 480,
///     "format": "PNG",
///     "timestamp": "00:01:30"
///   }
/// }
/// ```
pub async fn generate_thumbnail(
    State(state): State<AppState>,
    Json(payload): Json<ThumbnailRequest>,
) -> Result<Json<ThumbnailResponse>, (StatusCode, Json<ApiError>)> {
    // ---

    let pipeline_id = Uuid::new_v4().to_string();
    let timestamp = payload.timestamp.unwrap_or_else(|| "00:00:10".to_string());
    let width = payload.width.unwrap_or(320);
    let height = payload.height.unwrap_or(240);

    info!(
        "Generating thumbnail from: {} at {}",
        payload.source_url, timestamp
    );

    // Validate source URL
    if !payload.source_url.starts_with("http") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("Source URL must be a valid HTTP(S) URL")),
        ));
    }

    // Try to get media info to validate it's actually video content
    match get_media_info(&payload.source_url) {
        Ok(media_info) => {
            if media_info.width.is_none() || media_info.height.is_none() {
                warn!("Source may not be video content - proceeding anyway");
            } else {
                info!(
                    "Source video resolution: {}x{}",
                    media_info.width.unwrap_or(0),
                    media_info.height.unwrap_or(0)
                );
            }
        }
        Err(e) => {
            warn!("Could not analyze source for thumbnail: {}", e);
        }
    }

    // Create output path
    let output_path = format!("thumb_{pipeline_id}.png");

    // Use validation service to create thumbnail pipeline
    let pipeline_string =
        create_thumbnail_pipeline(&payload.source_url, &output_path, width, height, &timestamp);

    // Validate the generated pipeline
    if let Err(validation_error) = validate_pipeline_string(&pipeline_string) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Generated invalid thumbnail pipeline",
                &validation_error,
            )),
        ));
    }

    // Store pipeline info
    let pipeline_info = PipelineInfo {
        id: pipeline_id.clone(),
        description: "Generate thumbnail".to_string(),
        state: PipelineState::Created,
        pipeline_string,
        created_at: Utc::now().to_rfc3339(),
        source_url: Some(payload.source_url),
    };

    {
        let mut pipelines = state.lock().unwrap();
        pipelines.insert(pipeline_id.clone(), pipeline_info);
    }

    Ok(Json(ThumbnailResponse {
        pipeline_id,
        status: "created".to_string(),
        message: "Thumbnail generation initiated".to_string(),
        output_info: Some(ThumbnailInfo {
            width,
            height,
            format: "PNG".to_string(),
            timestamp,
        }),
    }))
}

/// Creates an adaptive streaming pipeline for HTTP Live Streaming (HLS) delivery.
///
/// Converts source media into HLS format with segmented transport streams and
/// M3U8 playlists, enabling adaptive bitrate streaming for web browsers and
/// mobile devices. Optimized for live-like streaming experiences with automatic
/// segment management.
///
/// # Request Body
/// Expects a JSON payload specifying source URL and streaming format:
/// ```json
/// {
///   "source_url": "https://example.com/video.mp4",
///   "stream_type": "hls"
/// }
/// ```
///
/// # Supported Streaming Formats
/// - **hls**: HTTP Live Streaming with M3U8 playlists and TS segments
/// - **dash**: MPEG-DASH (planned for future implementation)
/// - **rtmp**: Real-Time Messaging Protocol (planned for future implementation)
///
/// # HLS Stream Characteristics
/// - **Codec**: H.264 video encoding at 1000 kbps bitrate
/// - **Container**: MPEG Transport Stream (.ts) segments
/// - **Playlist**: M3U8 format compatible with HTML5 video players
/// - **Segment Management**: Rolling window of 10 segments maximum
/// - **Compatibility**: Works with iOS, Android, and modern web browsers
///
/// # Stream Architecture
/// ```text
/// Source Media → Decode → Encode (H.264) → Segment → HLS Output
///                                            ↓
///                              segment_00001.ts, segment_00002.ts, ...
///                                            ↓
///                                      playlist.m3u8
/// ```
///
/// # Validation and Setup
/// 1. **URL Protocol Validation**: Ensures HTTP(S) source URLs
/// 2. **Stream Type Validation**: Verifies supported streaming format
/// 3. **Pipeline Generation**: Creates optimized HLS streaming pipeline
/// 4. **Directory Preparation**: Sets up output directory structure
/// 5. **URL Generation**: Provides stream access URL for clients
///
/// # Response Behavior
/// - **200 OK**: Streaming pipeline created with access URL
/// - **400 Bad Request**: Invalid source URL or unsupported stream type
/// - **500 Internal Server Error**: Pipeline generation failure
///
/// # Client Integration
/// The returned stream URL can be used directly with:
/// - HTML5 `<video>` elements with HLS.js
/// - iOS and Android native video players
/// - Video.js and other web video libraries
/// - Broadcasting software and media servers
///
/// # Example Usage
/// ```bash
/// curl -X POST http://localhost:8080/stream \
///   -H "Content-Type: application/json" \
///   -d '{
///     "source_url": "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
///     "stream_type": "hls"
///   }'
/// ```
///
/// # Response Example
/// ```json
/// {
///   "pipeline_id": "550e8400-e29b-41d4-a716-446655440002",
///   "status": "created",
///   "stream_url": "http://localhost:8080/stream/550e8400-e29b-41d4-a716-446655440002/playlist.m3u8",
///   "message": "HLS stream created successfully"
/// }
/// ```
pub async fn create_stream(
    State(state): State<AppState>,
    Json(payload): Json<StreamRequest>,
) -> Result<Json<StreamResponse>, (StatusCode, Json<ApiError>)> {
    // ---

    let pipeline_id = Uuid::new_v4().to_string();

    info!(
        "Creating {} stream from: {}",
        payload.stream_type, payload.source_url
    );

    // Validate source URL
    if !payload.source_url.starts_with("http") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("Source URL must be a valid HTTP(S) URL")),
        ));
    }

    // Validate supported stream types
    if payload.stream_type != "hls" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new(
                "Unsupported stream type. Currently supported: hls",
            )),
        ));
    }

    // Create output directory path
    let output_dir = format!("stream_{pipeline_id}");

    // Use validation service to create streaming pipeline
    let pipeline_string = create_hls_stream_pipeline(&payload.source_url, &output_dir);

    // Validate the generated pipeline
    if let Err(validation_error) = validate_pipeline_string(&pipeline_string) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Generated invalid streaming pipeline",
                &validation_error,
            )),
        ));
    }

    // Store pipeline info
    let pipeline_info = PipelineInfo {
        id: pipeline_id.clone(),
        description: format!("{} streaming", payload.stream_type.to_uppercase()),
        state: PipelineState::Created,
        pipeline_string,
        created_at: Utc::now().to_rfc3339(),
        source_url: Some(payload.source_url),
    };

    {
        let mut pipelines = state.lock().unwrap();
        pipelines.insert(pipeline_id.clone(), pipeline_info);
    }

    let stream_url = Some(format!(
        "http://localhost:8080/stream/{pipeline_id}/playlist.m3u8",
    ));

    Ok(Json(StreamResponse {
        pipeline_id,
        status: "created".to_string(),
        stream_url,
        message: format!(
            "{} stream created successfully",
            payload.stream_type.to_uppercase()
        ),
    }))
}
