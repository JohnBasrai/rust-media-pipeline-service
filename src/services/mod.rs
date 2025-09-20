// EMBP Services Gateway: Controls public API for all service functionality
mod validation;

#[derive(Debug)]
pub struct MediaInfo {
    pub duration: Option<u64>, // in seconds
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bitrate: Option<u32>,
    pub format: String,
}

// Public exports - this defines the entire public services API
pub use validation::{
    create_conversion_pipeline, create_hls_stream_pipeline, create_thumbnail_pipeline,
    get_media_info, validate_pipeline_string,
};
