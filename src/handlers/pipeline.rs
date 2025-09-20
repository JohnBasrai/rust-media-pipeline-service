use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info; // error,
use uuid::Uuid;

// Import through gateways
use crate::models::{ApiError, CreatePipelineRequest, PipelineInfo, PipelineState};

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

    // Validate the pipeline string (basic validation)
    if payload.pipeline.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("Pipeline string cannot be empty")),
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
