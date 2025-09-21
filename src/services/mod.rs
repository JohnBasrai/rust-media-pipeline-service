//! Business logic and service layer for media processing operations.
//!
//! This module serves as the gateway for all business logic and external service
//! integrations, implementing the Explicit Module Boundary Pattern (EMBP) to
//! provide a clean interface between the HTTP handlers and the underlying
//! GStreamer multimedia framework.
//!
//! # Service Architecture
//!
//! The services layer abstracts complex GStreamer operations into simple,
//! type-safe functions that can be easily consumed by HTTP handlers. This
//! separation ensures that GStreamer-specific logic remains isolated and
//! that the API layer can focus on HTTP concerns.
//!
//! # Core Responsibilities
//!
//! - **Pipeline Validation**  : Ensuring GStreamer pipeline strings are syntactically correct
//! - **Media Analysis**       : Extracting metadata and technical information from media files  
//! - **Pipeline Construction**: Generating optimized pipelines for common operations
//! - **Error Translation**    : Converting GStreamer errors into application-level errors
//!
//! # EMBP Implementation
//!
//! This gateway module controls exactly which services and data structures
//! are exposed to handlers, maintaining clear boundaries between the business
//! logic layer and the presentation layer.

// ---

// EMBP Services Gateway: Controls public API for all service functionality
mod validation;

/// Media file metadata and technical information.
///
/// Contains comprehensive information about a media file extracted through
/// GStreamer discovery operations. This structure is used to communicate
/// media characteristics between the analysis services and API responses.
///
/// # Field Descriptions
/// - **duration**    : Length of the media in seconds (None if undetermined)
/// - **width/height**: Video dimensions in pixels (None for audio-only media)
/// - **bitrate**     : Data rate in bits per second (None if not available)
/// - **format**      : MIME type or container format identifier
///
/// # Usage Context
/// - Returned by media analysis endpoints
/// - Used internally for pipeline optimization decisions
/// - Provides clients with technical details for informed processing choices
#[derive(Debug)]
pub struct MediaInfo {
    // ---
    /// Duration of the media file in seconds
    pub duration: Option<u64>,

    /// Width of video content in pixels (None for audio-only)
    pub width: Option<u32>,

    /// Height of video content in pixels (None for audio-only)  
    pub height: Option<u32>,

    /// Bitrate of the media stream in bits per second
    pub bitrate: Option<u32>,

    /// Format identifier (MIME type or container format)
    pub format: String,
}

// ---

// Public exports - this defines the entire public services API
pub use validation::{
    create_conversion_pipeline, create_hls_stream_pipeline, create_thumbnail_pipeline,
    get_media_info, validate_pipeline_string,
};
