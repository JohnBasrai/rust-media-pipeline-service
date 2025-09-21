//! Response data transfer objects for the Media Pipeline Service API.
//!
//! This module defines all response structures returned by the REST API endpoints,
//! including success responses for various operations and standardized error handling.
//! All response types implement Serialize for JSON API compatibility.
//!
//! # Response Categories
//!
//! - **Operation Responses**: Status and metadata for initiated operations
//! - **Information Responses**: Data about media files and service capabilities  
//! - **Error Responses**: Standardized error information with optional details

use serde::Serialize;

/// Response returned after initiating a media format conversion operation.
///
/// This response indicates that a conversion pipeline has been created and queued
/// for execution. The actual conversion runs asynchronously, and clients should
/// poll the pipeline status endpoint to monitor progress.
///
/// # Example Response
/// ```json
/// {
///   "pipeline_id": "550e8400-e29b-41d4-a716-446655440000",
///   "status": "created",
///   "message": "Conversion to webm initiated",
///   "estimated_duration": "2-5 minutes"
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct ConvertResponse {
    // ---
    /// Unique identifier for the created conversion pipeline
    pub pipeline_id: String,

    /// Current status of the conversion request (typically "created")
    pub status: String,

    /// Human-readable description of the operation status
    pub message: String,

    /// Optional estimate of how long the conversion might take
    pub estimated_duration: Option<String>,
}

/// Response returned after initiating a thumbnail generation operation.
///
/// Contains the pipeline ID for tracking the thumbnail generation process
/// along with metadata about the requested thumbnail specifications.
///
/// # Example Response
/// ```json
/// {
///   "pipeline_id": "550e8400-e29b-41d4-a716-446655440001",
///   "status": "created",
///   "message": "Thumbnail generation initiated",
///   "output_info": {
///     "width": 640,
///     "height": 480,
///     "format": "PNG",
///     "timestamp": "00:01:30"
///   }
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct ThumbnailResponse {
    // ---
    /// Unique identifier for the created thumbnail pipeline
    pub pipeline_id: String,

    /// Current status of the thumbnail request (typically "created")
    pub status: String,

    /// Human-readable description of the operation status
    pub message: String,

    /// Optional details about the thumbnail specifications
    pub output_info: Option<ThumbnailInfo>,
}

/// Detailed information about a generated thumbnail's specifications.
///
/// Provides the exact dimensions, format, and timestamp used for thumbnail
/// extraction from the source video.
#[derive(Debug, Serialize)]
pub struct ThumbnailInfo {
    // ---
    /// Width of the generated thumbnail in pixels
    pub width: u32,

    /// Height of the generated thumbnail in pixels
    pub height: u32,

    /// Image format of the thumbnail (e.g., "PNG", "JPEG")
    pub format: String,

    /// Timestamp in the source video where thumbnail was extracted (HH:MM:SS format)
    pub timestamp: String,
}

/// Response returned after creating a streaming pipeline.
///
/// Contains information about the created streaming pipeline and the URL
/// where the stream will be accessible once processing begins.
///
/// # Example Response
/// ```json
/// {
///   "pipeline_id": "550e8400-e29b-41d4-a716-446655440002",
///   "status": "created",
///   "stream_url": "http://localhost:8080/stream/550e8400-e29b-41d4-a716-446655440002/playlist.m3u8",
///   "message": "HLS stream created successfully"
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct StreamResponse {
    // ---
    /// Unique identifier for the created streaming pipeline
    pub pipeline_id: String,

    /// Current status of the streaming request (typically "created")
    pub status: String,

    /// Optional URL where the stream will be accessible (for HLS: .m3u8 playlist)
    pub stream_url: Option<String>,

    /// Human-readable description of the operation status
    pub message: String,
}

/// Information about a sample media file available for testing.
///
/// The service provides several sample media files that can be used to test
/// various API endpoints without needing to provide your own media URLs.
///
/// # Example Response
/// ```json
/// {
///   "name": "Big Buck Bunny",
///   "url": "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
///   "media_type": "video",
///   "duration": "10:34",
///   "description": "Blender Foundation's famous open-source short film"
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct SampleMedia {
    // ---
    /// Human-readable name of the sample media
    pub name: String,

    /// Direct URL to the media file
    pub url: String,

    /// Type of media content ("video" or "audio")
    pub media_type: String,

    /// Optional duration of the media in MM:SS or HH:MM:SS format
    pub duration: Option<String>,

    /// Description of the media content and its source
    pub description: String,
}

/// Standardized error response structure for all API endpoints.
///
/// Provides consistent error reporting across the API with optional additional
/// details for debugging. All API errors return HTTP status codes with this
/// JSON structure in the response body.
///
/// # Example Error Response
/// ```json
/// {
///   "error": "Invalid pipeline configuration",
///   "details": "Pipeline must contain at least one element connection (!)"
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct ApiError {
    // ---
    /// High-level error message suitable for display to users
    pub error: String,

    /// Optional additional technical details about the error
    pub details: Option<String>,
}

impl ApiError {
    // ---

    /// Creates a new ApiError with just an error message.
    ///
    /// # Arguments
    /// * `error` - The main error message to display
    ///
    /// # Example
    /// ```rust
    /// let error = ApiError::new("Pipeline not found");
    /// ```
    pub fn new(error: &str) -> Self {
        // ---
        Self {
            error: error.to_string(),
            details: None,
        }
    }

    /// Creates a new ApiError with both an error message and additional details.
    ///
    /// Use this when you have technical details that might help with debugging
    /// or when the error originates from an underlying system (like GStreamer).
    ///
    /// # Arguments
    /// * `error` - The main error message to display
    /// * `details` - Additional technical information about the error
    ///
    /// # Example
    /// ```rust
    /// let error = ApiError::with_details(
    ///     "Failed to analyze media",
    ///     "GStreamer error: Could not determine file format"
    /// );
    /// ```
    pub fn with_details(error: &str, details: &str) -> Self {
        // ---
        Self {
            error: error.to_string(),
            details: Some(details.to_string()),
        }
    }
}
