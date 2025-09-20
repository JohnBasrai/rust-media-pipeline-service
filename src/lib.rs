// Library interface for media-pipeline-service
// Re-export everything needed for integration tests

pub mod handlers;
pub mod models;
mod services;

// Re-export the key types and functions needed by integration tests
pub use handlers::{analyze_media, AppState};

pub use models::{
    // ---
    ConvertRequest,
    ConvertResponse,
    PipelineInfo,
    PipelineState,
    StreamRequest,
    StreamResponse,
    ThumbnailRequest,
    ThumbnailResponse,
};

pub use services::PipelineService;
