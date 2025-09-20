use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ConvertResponse {
    pub pipeline_id: String,
    pub status: String,
    pub message: String,
    pub estimated_duration: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ThumbnailResponse {
    pub pipeline_id: String,
    pub status: String,
    pub message: String,
    pub output_info: Option<ThumbnailInfo>,
}

#[derive(Debug, Serialize)]
pub struct ThumbnailInfo {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct StreamResponse {
    pub pipeline_id: String,
    pub status: String,
    pub stream_url: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SampleMedia {
    pub name: String,
    pub url: String,
    pub media_type: String, // "video", "audio"
    pub duration: Option<String>,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub details: Option<String>,
}

impl ApiError {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
            details: None,
        }
    }

    pub fn with_details(error: &str, details: &str) -> Self {
        Self {
            error: error.to_string(),
            details: Some(details.to_string()),
        }
    }
}
