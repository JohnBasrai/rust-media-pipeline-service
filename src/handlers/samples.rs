//! Sample media and health check endpoints for the Media Pipeline Service.
//!
//! This module provides endpoints that support testing, service discovery, and
//! operational monitoring. It includes a curated collection of public domain
//! media files for testing API functionality and a comprehensive health check
//! endpoint for service monitoring.
//!
//! # Endpoint Categories
//!
//! - **Health Monitoring**: Service status and capability reporting
//! - **Sample Media**     : Curated test media files with known characteristics
//! - **Service Discovery**: API endpoint enumeration and documentation
//!
//! # Sample Media Curation
//!
//! All sample media files are carefully selected to be:
//! - Publicly available and legally distributable
//! - Diverse in format and characteristics for comprehensive testing
//! - Stable URLs that remain accessible over time
//! - Representative of real-world media processing scenarios

use axum::response::Json;

// ---

// Import through gateway
use crate::models::SampleMedia;

/// Curated collection of sample media files for API testing and demonstration.
///
/// Contains a diverse set of media files with different formats, durations, and
/// characteristics. All files are from public domain sources or open content
/// initiatives, ensuring they can be freely used for testing purposes.
///
/// # Media Selection Criteria
/// - **Legal**: Public domain or Creative Commons licensed content
/// - **Stable**: Hosted on reliable infrastructure with persistent URLs
/// - **Diverse**: Various formats, durations, and quality levels
/// - **Representative**: Real-world content suitable for testing scenarios
///
/// # Format Coverage
/// - **Video**: MP4 container with H.264 encoding (web-compatible)
/// - **Audio**: MP3 format for broad compatibility testing
/// - **Duration Range**: From 30 seconds to 10+ minutes for different use cases
const SAMPLE_MEDIA: &[(&str, &str, &str, &str, &str)] = &[
    (
        "Big Buck Bunny",
        "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
        "video",
        "10:34",
        "Blender Foundation's famous open-source short film",
    ),
    (
        "Elephant's Dream",
        "https://archive.org/download/ElephantsDream/ed_hd.mp4",
        "video",
        "10:53",
        "First open movie made entirely with Free Software",
    ),
    (
        "Classical Music Sample",
        "https://archive.org/download/testmp3testfile/mpthreetest.mp3",
        "audio",
        "0:30",
        "Public domain classical music sample",
    ),
    (
        "Nature Documentary",
        "https://archive.org/download/night-15441/night-15441.mp4",
        "video",
        "2:15",
        "Public domain nature footage",
    ),
];

/// Lists all available sample media files for testing API endpoints.
///
/// Returns a collection of curated media files that can be used to test various
/// API functionality without requiring clients to provide their own media URLs.
/// Each sample includes metadata about format, duration, and content description.
///
/// # Response Format
/// Returns an array of `SampleMedia` objects, each containing:
/// - Media file name and description
/// - Direct URL to the media file
/// - Media type classification (video/audio)
/// - Duration information when available
/// - Content description and source attribution
///
/// # Use Cases
/// - **API Testing**: Verify endpoint functionality with known-good media
/// - **Integration Testing**: Automated testing with reliable media sources
/// - **Demo Purposes**: Showcase service capabilities without content preparation
/// - **Format Validation**: Test with diverse media characteristics
///
/// # Example Usage
/// ```bash
/// curl http://localhost:8080/samples
/// ```
///
/// # Response Example
/// ```json
/// [
///   {
///     "name": "Big Buck Bunny",
///     "url": "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
///     "media_type": "video",
///     "duration": "10:34",
///     "description": "Blender Foundation's famous open-source short film"
///   }
/// ]
/// ```
pub async fn list_sample_media() -> Json<Vec<SampleMedia>> {
    // ---

    let samples = SAMPLE_MEDIA
        .iter()
        .map(
            |(name, url, media_type, duration, description)| SampleMedia {
                name: name.to_string(),
                url: url.to_string(),
                media_type: media_type.to_string(),
                duration: Some(duration.to_string()),
                description: description.to_string(),
            },
        )
        .collect();

    Json(samples)
}

/// Comprehensive service health check and capability reporting endpoint.
///
/// Provides detailed information about service status, underlying framework
/// versions, and available API endpoints. This endpoint is designed for both
/// automated monitoring systems and human operators to quickly assess service
/// health and capabilities.
///
/// # Health Check Components
/// - **Service Status**: Overall operational state
/// - **Framework Versions**: GStreamer and runtime version information
/// - **API Documentation**: Complete endpoint enumeration with descriptions
/// - **Capability Reporting**: Available media processing features
///
/// # Monitoring Integration
/// This endpoint is suitable for:
/// - **Load Balancer Health Checks**: Simple up/down status determination
/// - **Service Discovery**: Automated discovery of available capabilities
/// - **Operational Dashboards**: Human-readable service status displays
/// - **Dependency Verification**: Confirmation of required framework availability
///
/// # Response Structure
/// - **status**: Simple "healthy" indicator for automated systems
/// - **service**: Service identification and version information
/// - **gstreamer_version**: Underlying GStreamer framework version
/// - **endpoints**: Complete API documentation with method and description
///
/// # Example Usage
/// ```bash
/// # Basic health check
/// curl http://localhost:8080/health
///
/// # Root endpoint (alias for health check)
/// curl http://localhost:8080/
/// ```
///
/// # Response Example
/// ```json
/// {
///   "status": "healthy",
///   "service": "Rust Media Pipeline Service",
///   "gstreamer_version": "1.20.3",
///   "endpoints": [
///     "GET /health - Health check",
///     "POST /convert - Convert media format",
///     "..."
///   ]
/// }
/// ```
pub async fn health_check() -> Json<serde_json::Value> {
    // ---

    Json(serde_json::json!({
        "status": "healthy",
        "service": "Rust Media Pipeline Service",
        "gstreamer_version": gstreamer::version_string().to_string(),
        "endpoints": [
            "GET /health - Health check",
            "GET /samples - List sample media",
            "POST /convert - Convert media format",
            "POST /thumbnail - Generate thumbnail",
            "POST /stream - Create streaming pipeline",
            "GET /pipelines - List active pipelines",
            "POST /pipelines - Create custom pipeline",
            "GET /pipelines/{id} - Get pipeline status",
            "DELETE /pipelines/{id} - Stop pipeline"
        ]
    }))
}
