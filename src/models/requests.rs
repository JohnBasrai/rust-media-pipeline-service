use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreatePipelineRequest {
    pub description: String,
    pub pipeline: String,
}

#[derive(Debug, Deserialize)]
pub struct ConvertRequest {
    pub source_url: String,
    pub output_format: String,   // "webm", "mp4", "avi"
    pub quality: Option<String>, // "high", "medium", "low"
}

#[derive(Debug, Deserialize)]
pub struct ThumbnailRequest {
    pub source_url: String,
    pub timestamp: Option<String>, // "00:01:30" format
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct StreamRequest {
    pub source_url: String,
    pub stream_type: String,     // "hls", "dash", "rtmp"
    pub bitrate: Option<String>, // "1000k", "2000k", etc.
}
