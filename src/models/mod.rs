// EMBP Models Gateway: Controls public API for all model types
mod pipeline;
mod requests;
mod responses;

// Public exports - this defines the entire public models API
pub use pipeline::{PipelineInfo, PipelineState};
pub use requests::{ConvertRequest, CreatePipelineRequest, StreamRequest, ThumbnailRequest};
pub use responses::{
    ApiError, ConvertResponse, SampleMedia, StreamResponse, ThumbnailInfo, ThumbnailResponse,
};
