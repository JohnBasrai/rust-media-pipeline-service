use axum::{extract::State, http::StatusCode, response::Json};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info;
use uuid::Uuid;

// Import through gateways
use crate::models::{
    ApiError, ConvertRequest, ConvertResponse, PipelineInfo, PipelineState, StreamRequest,
    StreamResponse, ThumbnailInfo, ThumbnailRequest, ThumbnailResponse,
};

// Type alias for shared state
type AppState = Arc<Mutex<HashMap<String, PipelineInfo>>>;

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

    // Create conversion pipeline string based on output format
    let pipeline_string = match payload.output_format.as_str() {
        "webm" => format!(
            "souphttpsrc location={} ! decodebin ! videoconvert ! vp8enc ! webmmux ! filesink location=output_{}.webm",
            payload.source_url, pipeline_id
        ),
        "mp4" => format!(
            "souphttpsrc location={} ! decodebin ! videoconvert ! x264enc ! mp4mux ! filesink location=output_{}.mp4", 
            payload.source_url, pipeline_id
        ),
        _ => return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("Unsupported output format. Use: webm, mp4"))
        ))
    };

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

    // Create thumbnail pipeline
    let pipeline_string = format!(
        "souphttpsrc location={} ! decodebin ! videoconvert ! videoscale ! video/x-raw,width={},height={} ! pngenc ! filesink location=thumb_{}.png",
        payload.source_url, width, height, pipeline_id
    );

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
    // ---
    let pipeline_id = Uuid::new_v4().to_string();

    info!(
        "Creating {} stream from: {}",
        payload.stream_type, payload.source_url
    );

    let pipeline_string = match payload.stream_type.as_str() {
        "hls" => format!(
            "souphttpsrc location={} ! decodebin ! videoconvert ! x264enc ! mpegtsmux ! hlssink location=stream_{}/segment_%05d.ts playlist-location=stream_{}/playlist.m3u8",
            payload.source_url, pipeline_id, pipeline_id
        ),
        _ => return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("Unsupported stream type. Use: hls"))
        ))
    };

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
