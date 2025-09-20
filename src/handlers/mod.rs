// EMBP Handlers Gateway: Controls public API for all handler functions
mod media;
mod pipeline;
mod samples;

// Public exports - this defines the entire public handlers API
pub use media::{convert_media, create_stream, generate_thumbnail, AppState};
pub use pipeline::{analyze_media, create_pipeline, get_pipeline, list_pipelines, stop_pipeline};
pub use samples::{health_check, list_sample_media};
