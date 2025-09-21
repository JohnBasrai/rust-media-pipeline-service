//! Data models and transfer objects for the Media Pipeline Service.
//!
//! This module serves as the gateway for all data structures used throughout
//! the application, implementing the Explicit Module Boundary Pattern (EMBP)
//! to control public API exposure and maintain clean separation of concerns.
//!
//! # Module Organization
//!
//! The models are organized into three logical categories:
//! - **Pipeline Models** : Core pipeline state management and metadata
//! - **Request Models**  : Input DTOs for API endpoints accepting JSON payloads
//! - **Response Models** : Output DTOs for API responses and error handling
//!
//! # EMBP Implementation
//!
//! This gateway module controls exactly which types are exposed to the rest
//! of the application. Internal implementation details remain private while
//! providing a clean, stable API for consumers.
//!
//! # Serialization
//!
//! All models implement appropriate Serde traits:
//! - Request models implement `Deserialize` for JSON input parsing
//! - Response models implement `Serialize` for JSON output generation
//! - Pipeline models implement both for full API compatibility

// ---

// EMBP Models Gateway: Controls public API for all model types
mod pipeline;
mod requests;
mod responses;

// ---

// Public exports - this defines the entire public models API
pub use pipeline::{PipelineInfo, PipelineState};
pub use requests::{ConvertRequest, CreatePipelineRequest, StreamRequest, ThumbnailRequest};
pub use responses::{
    ApiError, ConvertResponse, SampleMedia, StreamResponse, ThumbnailInfo, ThumbnailResponse,
};
