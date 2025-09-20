use axum::{extract::State, http::StatusCode, response::Json};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};
use uuid::Uuid;

// Import through gateways
use crate::models::{
    ApiError, ConvertRequest, ConvertResponse, PipelineInfo, PipelineState, StreamRequest,
    StreamResponse, ThumbnailInfo, ThumbnailRequest, ThumbnailResponse,
};
use crate::services::{
    create_conversion_pipeline, create_hls_stream_pipeline, create_thumbnail_pipeline,
    get_media_info, validate_pipeline_string,
};

// Type alias for shared state
pub type AppState = Arc<Mutex<HashMap<String, PipelineInfo>>>;

pub async fn convert_media(
    State(state): State<AppState>,
    Json(payload): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, (StatusCode, Json<ApiError>)> {
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

pub async fn generate_thumbnail(
    State(state): State<AppState>,
    Json(payload): Json<ThumbnailRequest>,
) -> Result<Json<ThumbnailResponse>, (StatusCode, Json<ApiError>)> {
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

pub async fn create_stream(
    State(state): State<AppState>,
    Json(payload): Json<StreamRequest>,
) -> Result<Json<StreamResponse>, (StatusCode, Json<ApiError>)> {
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
