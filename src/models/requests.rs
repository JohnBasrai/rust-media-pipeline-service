//! Request data transfer objects for the Media Pipeline Service API.
//!
//! This module defines all request structures accepted by the REST API endpoints.
//! These DTOs represent the input data for various media processing operations
//! and pipeline management functions. All request types implement Deserialize
//! for JSON API compatibility.
//!
//! # Request Categories
//!
//! - **Pipeline Creation**: Requests for creating custom GStreamer pipelines
//! - **Media Processing**: Requests for format conversion, thumbnails, and streaming
//! - **Validation**: All requests include implicit validation through type constraints

use serde::Deserialize;

/// Request to create a custom GStreamer pipeline.
///
/// Allows clients to submit custom GStreamer pipeline strings for execution.
/// The pipeline string is validated before execution to ensure it contains
/// valid GStreamer syntax and elements.
///
/// # Example Request
/// ```json
/// {
///   "description": "Audio extraction pipeline",
///   "pipeline": "souphttpsrc location=https://example.com/video.mp4 ! decodebin ! audioconvert ! vorbisenc ! oggmux ! filesink location=output.ogg"
/// }
/// ```
///
/// # Pipeline String Requirements
/// - Must contain valid GStreamer element names
/// - Elements must be connected with `!` operators
/// - Source and sink elements should be properly configured
/// - Pipeline will be validated before execution
#[derive(Debug, Deserialize)]
pub struct CreatePipelineRequest {
    // ---
    /// Human-readable description of what this pipeline does
    pub description: String,

    /// Complete GStreamer pipeline string for execution
    pub pipeline: String,
}

/// Request to convert media between different formats.
///
/// Initiates a media format conversion operation using predefined GStreamer
/// pipelines optimized for common conversion scenarios. The service supports
/// conversion between WebM, MP4, and AVI formats.
///
/// # Example Request
/// ```json
/// {
///   "source_url": "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
///   "output_format": "webm"
/// }
/// ```
///
/// # Supported Formats
/// - **webm**: VP8 video codec with WebM container
/// - **mp4**: H.264 video codec with MP4 container  
/// - **avi**: H.264 video codec with AVI container
#[derive(Debug, Deserialize)]
pub struct ConvertRequest {
    // ---
    /// HTTP(S) URL of the source media file to convert
    pub source_url: String,

    /// Target output format ("webm", "mp4", "avi")
    pub output_format: String,
}

/// Request to generate a thumbnail image from a video source.
///
/// Extracts a single frame from a video at a specified timestamp and converts
/// it to a PNG image with optional resizing. If dimensions are not specified,
/// default values are used (320x240).
///
/// # Example Request
/// ```json
/// {
///   "source_url": "https://example.com/video.mp4",
///   "timestamp": "00:01:30",
///   "width": 640,
///   "height": 480
/// }
/// ```
///
/// # Timestamp Format
/// - Accepts HH:MM:SS format (e.g., "00:01:30" for 1 minute 30 seconds)
/// - If not provided, defaults to "00:00:10" (10 seconds into the video)
/// - Should be within the video's actual duration
#[derive(Debug, Deserialize)]
pub struct ThumbnailRequest {
    // ---
    /// HTTP(S) URL of the source video file
    pub source_url: String,

    /// Optional timestamp to extract thumbnail from (HH:MM:SS format)
    /// Defaults to "00:00:10" if not provided
    pub timestamp: Option<String>,

    /// Optional width of the generated thumbnail in pixels
    /// Defaults to 320 if not provided
    pub width: Option<u32>,

    /// Optional height of the generated thumbnail in pixels  
    /// Defaults to 240 if not provided
    pub height: Option<u32>,
}

/// Request to create a streaming pipeline.
///
/// Sets up a streaming pipeline that converts a source media file into
/// a format suitable for adaptive streaming. Currently supports HLS
/// (HTTP Live Streaming) with plans for DASH and RTMP support.
///
/// # Example Request
/// ```json
/// {
///   "source_url": "https://example.com/video.mp4",
///   "stream_type": "hls"
/// }
/// ```
///
/// # Supported Stream Types
/// - **hls**: HTTP Live Streaming with .m3u8 playlists and .ts segments
/// - **dash**: MPEG-DASH (planned for future implementation)
/// - **rtmp**: Real-Time Messaging Protocol (planned for future implementation)
#[derive(Debug, Deserialize)]
pub struct StreamRequest {
    // ---
    /// HTTP(S) URL of the source media file to stream
    pub source_url: String,

    /// Type of streaming format to create ("hls", "dash", "rtmp")
    /// Currently only "hls" is fully supported
    pub stream_type: String,
}
