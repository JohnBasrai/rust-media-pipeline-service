// EMBP Services Gateway: Controls public API for all service functionality
mod pipeline;
mod validation;

// Public exports - this defines the entire public services API
pub use pipeline::{MediaInfo, PipelineService};
pub use validation::{
    create_conversion_pipeline, create_hls_stream_pipeline, create_thumbnail_pipeline,
    get_media_info, validate_pipeline_string,
};
