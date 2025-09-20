use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};
use uuid::Uuid;

// Import through gateways
use crate::models::{ApiError, CreatePipelineRequest, PipelineInfo, PipelineState};
use crate::services::{get_media_info, validate_pipeline_string};

// Type alias for shared state
type AppState = Arc<Mutex<HashMap<String, PipelineInfo>>>;

pub async fn create_pipeline(
    State(state): State<AppState>,
    Json(payload): Json<CreatePipelineRequest>,
) -> Result<Json<PipelineInfo>, (StatusCode, Json<ApiError>)> {
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

pub async fn list_pipelines(State(state): State<AppState>) -> Json<Vec<PipelineInfo>> {
    let pipelines = state.lock().unwrap();
    let pipeline_list: Vec<PipelineInfo> = pipelines.values().cloned().collect();
    Json(pipeline_list)
}

pub async fn get_pipeline(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PipelineInfo>, (StatusCode, Json<ApiError>)> {
    let pipelines = state.lock().unwrap();

    match pipelines.get(&id) {
        Some(pipeline) => Ok(Json(pipeline.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("Pipeline not found")),
        )),
    }
}

pub async fn stop_pipeline(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
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

// New endpoint to analyze media before processing
pub async fn analyze_media(
    Path(url): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
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
