//! GStreamer pipeline validation and media processing services.
//!
//! This module provides the core business logic for validating GStreamer pipeline
//! strings, analyzing media files, and constructing common pipeline patterns.
//! It serves as the bridge between the HTTP API layer and the GStreamer multimedia
//! framework, ensuring all operations are safe and properly validated.
//!
//! # Core Functionality
//!
//! - **Pipeline Validation**  : Syntax checking and element verification for custom pipelines
//! - **Media Discovery**      : Analysis of remote media files to extract metadata
//! - **Pipeline Construction**: Programmatic generation of common pipeline patterns
//! - **Error Handling**       : Comprehensive error reporting from GStreamer operations
//!
//! # GStreamer Integration
//!
//! All functions interact directly with the GStreamer framework and require
//! proper GStreamer initialization before use. Pipeline validation uses
//! GStreamer's built-in parsing to catch syntax errors early.

use gstreamer::prelude::*;
use std::time::Duration;

// ---

// Import from parent module
use super::MediaInfo;

/// Validates a GStreamer pipeline string for syntax and basic structural correctness.
///
/// Performs comprehensive validation including syntax checking, element connectivity
/// verification, and GStreamer parsing validation. This function should be called
/// before attempting to execute any custom pipeline to prevent runtime errors.
///
/// # Arguments
/// * `pipeline_string` - The complete GStreamer pipeline string to validate
///
/// # Returns
/// * `Ok(())` - Pipeline is valid and can be executed
/// * `Err(String)` - Validation failed with detailed error message
///
/// # Validation Checks
/// - Pipeline string is not empty or whitespace-only
/// - Contains proper element connections (!) between components
/// - GStreamer can successfully parse the pipeline syntax
/// - All referenced elements are available in the current GStreamer installation
///
/// # Example
/// ```rust
/// // Valid pipeline
/// assert!(validate_pipeline_string("fakesrc ! fakesink").is_ok());
///
/// // Invalid - missing connections
/// assert!(validate_pipeline_string("fakesrc fakesink").is_err());
///
/// // Invalid - empty string
/// assert!(validate_pipeline_string("").is_err());
/// ```
pub fn validate_pipeline_string(pipeline_string: &str) -> Result<(), String> {
    // ---

    // Basic validation - check for common GStreamer elements
    if pipeline_string.trim().is_empty() {
        return Err("Pipeline string cannot be empty".to_string());
    }

    // Check for basic pipeline structure (elements connected with !)
    if !pipeline_string.contains('!') {
        return Err("Pipeline must contain at least one element connection (!)".to_string());
    }

    // Try to parse the pipeline to catch syntax errors
    match gstreamer::parse_launch(pipeline_string) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Invalid pipeline syntax: {e}")),
    }
}

/// Analyzes a remote media file to extract format, duration, and technical metadata.
///
/// Creates a temporary GStreamer discovery pipeline to probe the media file
/// without fully downloading or decoding it. This function attempts to gather
/// as much information as possible about the media file's characteristics.
///
/// # Arguments
/// * `url` - HTTP(S) URL of the media file to analyze
///
/// # Returns
/// * `Ok(MediaInfo)` - Successfully extracted media information
/// * `Err(anyhow::Error)` - Failed to analyze media (network, format, or GStreamer errors)
///
/// # Extracted Information
/// - **Duration**  : Length of the media file in seconds
/// - **Dimensions**: Width and height for video content
/// - **Format**    : MIME type or container format
/// - **Bitrate**   : Data rate (when available)
///
/// # Implementation Details
/// - Uses a discovery pipeline that pauses at PAUSED state for analysis
/// - Implements 10-second timeout to prevent hanging on unresponsive sources
/// - Falls back to URL-based format detection when caps negotiation fails
/// - Properly cleans up GStreamer resources after analysis
///
/// # Example
/// ```rust
/// let info = get_media_info("https://example.com/video.mp4")?;
/// println!("Duration: {} seconds", info.duration.unwrap_or(0));
/// println!("Format: {}", info.format);
/// ```
pub fn get_media_info(url: &str) -> anyhow::Result<MediaInfo> {
    // ---

    use gstreamer::MessageView;

    // Create a discovery pipeline - we'll probe the media without fully decoding
    let pipeline_string = format!(
        "souphttpsrc location={url} ! typefind ! identity signal-handoffs=false ! fakesink sync=false"
    );

    let pipeline = gstreamer::parse_launch(&pipeline_string)?
        .downcast::<gstreamer::Pipeline>()
        .map_err(|_| anyhow::anyhow!("Failed to create discovery pipeline"))?;

    // Set to PAUSED state to trigger caps negotiation without playing
    pipeline.set_state(gstreamer::State::Paused)?;

    // Get the bus to listen for messages
    let bus = pipeline.bus().expect("Pipeline without bus");

    let mut media_info = MediaInfo {
        duration: None,
        width: None,
        height: None,
        bitrate: None,
        format: "unknown".to_string(),
    };

    // Wait for state change to PAUSED or error (with timeout)
    let timeout = Duration::from_secs(10);
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < timeout {
        if let Some(msg) = bus.timed_pop(gstreamer::ClockTime::from_mseconds(100)) {
            match msg.view() {
                MessageView::Error(err) => {
                    pipeline.set_state(gstreamer::State::Null)?;
                    return Err(anyhow::anyhow!("Pipeline error: {}", err.error()));
                }
                MessageView::StateChanged(state_changed) => {
                    if state_changed.src().map(|s| s == &pipeline).unwrap_or(false)
                        && state_changed.current() == gstreamer::State::Paused
                    {
                        // Pipeline is now paused, we can query information
                        break;
                    }
                }
                MessageView::AsyncDone(_) => {
                    // Pipeline has finished transitioning to PAUSED
                    break;
                }
                _ => {}
            }
        }
    }

    // Try to get duration
    if let Some(duration) = pipeline.query_duration::<gstreamer::ClockTime>() {
        media_info.duration = Some(duration.seconds());
    }

    // Try to get format information from the typefind element
    if let Some(typefind) = pipeline.by_name("typefind0") {
        if let Some(caps) = typefind
            .static_pad("src")
            .and_then(|pad| pad.current_caps())
        {
            if let Some(structure) = caps.structure(0) {
                media_info.format = structure.name().to_string();

                // Try to get video dimensions if it's a video format
                if let Ok(width) = structure.get::<i32>("width") {
                    media_info.width = Some(width as u32);
                }
                if let Ok(height) = structure.get::<i32>("height") {
                    media_info.height = Some(height as u32);
                }
            }
        }
    }

    // Alternative: try to find any video pad in the pipeline and get its caps
    if media_info.width.is_none() || media_info.height.is_none() {
        for pad_result in pipeline.iterate_pads().into_iter().flatten() {
            if let Some(caps) = pad_result.current_caps() {
                for i in 0..caps.size() {
                    if let Some(structure) = caps.structure(i) {
                        if structure.name().starts_with("video/") {
                            if let Ok(width) = structure.get::<i32>("width") {
                                media_info.width = Some(width as u32);
                            }
                            if let Ok(height) = structure.get::<i32>("height") {
                                media_info.height = Some(height as u32);
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    // Clean up
    pipeline.set_state(gstreamer::State::Null)?;

    // If we still don't have format info, try to infer from URL
    if media_info.format == "unknown" {
        if url.contains(".mp4") {
            media_info.format = "video/mp4".to_string();
        } else if url.contains(".webm") {
            media_info.format = "video/webm".to_string();
        } else if url.contains(".mp3") {
            media_info.format = "audio/mpeg".to_string();
        } else if url.contains(".ogg") {
            media_info.format = "audio/ogg".to_string();
        }
    }

    Ok(media_info)
}

/// Creates a GStreamer pipeline string for media format conversion.
///
/// Generates optimized pipeline configurations for converting between common
/// video formats. Each format uses appropriate codecs and containers for
/// broad compatibility and reasonable quality/file size trade-offs.
///
/// # Arguments
/// * `source_url` - HTTP(S) URL of the source media file
/// * `output_format` - Target format ("webm", "mp4", "avi")
/// * `output_path` - Local filesystem path for the converted output file
///
/// # Returns
/// * `Ok(String)` - Complete GStreamer pipeline string ready for execution
/// * `Err(String)` - Unsupported format or configuration error
///
/// # Supported Conversions
/// - **webm**: VP8 video codec with WebM container (open source, web-optimized)
/// - **mp4** : H.264 video codec with MP4 container (broad compatibility)
/// - **avi** : H.264 video codec with AVI container (legacy compatibility)
///
/// # Pipeline Patterns
/// All conversion pipelines follow the same general structure:
/// `source → decode → convert → encode → mux → output`
///
/// # Example
/// ```rust
/// let pipeline = create_conversion_pipeline(
///     "https://example.com/input.mp4",
///     "webm",
///     "output.webm"
/// )?;
/// ```
pub fn create_conversion_pipeline(
    source_url: &str,
    output_format: &str,
    output_path: &str,
) -> Result<String, String> {
    // ---

    match output_format {
        "webm" => Ok(format!(
            "souphttpsrc location={source_url} ! decodebin ! videoconvert ! vp8enc ! webmmux ! filesink location={output_path}"
        )),
        "mp4" => Ok(format!(
            "souphttpsrc location={source_url} ! decodebin ! videoconvert ! x264enc ! mp4mux ! filesink location={output_path}"
        )),
        "avi" => Ok(format!(
            "souphttpsrc location={source_url} ! decodebin ! videoconvert ! x264enc ! avimux ! filesink location={output_path}"
        )),
        _ => Err(format!("Unsupported output format: {output_format}")),
    }
}

/// Creates a GStreamer pipeline string for thumbnail extraction from video.
///
/// Generates a pipeline that extracts a single frame from a video source at the
/// specified timestamp, scales it to the requested dimensions, and saves it as
/// a PNG image file.
///
/// # Arguments
/// * `source_url`  - HTTP(S) URL of the source video file
/// * `output_path` - Local filesystem path for the generated thumbnail
/// * `width`       - Width of the thumbnail in pixels
/// * `height`      - Height of the thumbnail in pixels
/// * `_timestamp`  - Target timestamp for frame extraction (currently unused in pipeline)
///
/// # Returns
/// A complete GStreamer pipeline string for thumbnail generation
///
/// # Pipeline Structure
/// `source → decode → convert → scale → encode → output`
///
/// # Notes
/// - Currently extracts from early in the video stream rather than exact timestamp
/// - Uses PNG format for lossless thumbnail quality
/// - Aspect ratio is not preserved - image is scaled to exact dimensions
/// - Future enhancement could implement precise seeking to timestamp
///
/// # Example
/// ```rust
/// let pipeline = create_thumbnail_pipeline(
///     "https://example.com/video.mp4",
///     "thumb.png",
///     640,
///     480,
///     "00:01:30"
/// );
/// ```
pub fn create_thumbnail_pipeline(
    source_url: &str,
    output_path: &str,
    width: u32,
    height: u32,
    _timestamp: &str,
) -> String {
    // ---

    format!(
        "souphttpsrc location={source_url} ! decodebin ! videoconvert ! videoscale ! video/x-raw,width={width},height={height} ! pngenc ! filesink location={output_path}"
    )
}

/// Creates a GStreamer pipeline string for HLS streaming.
///
/// Generates a pipeline that converts a source media file into HTTP Live Streaming
/// format with segmented transport stream files and an M3U8 playlist. The output
/// is suitable for adaptive streaming to web browsers and mobile devices.
///
/// # Arguments
/// * `source_url` - HTTP(S) URL of the source media file
/// * `output_dir` - Directory path where HLS segments and playlist will be created
///
/// # Returns
/// A complete GStreamer pipeline string for HLS streaming
///
/// # HLS Output Structure
/// - **Segments**: Individual .ts files containing media chunks
/// - **Playlist**: .m3u8 file listing segments and metadata
/// - **Bitrate**: Fixed at 1000 kbps for consistent streaming
/// - **Segment Management**: Keeps maximum of 10 segments (rolling window)
///
/// # Pipeline Structure
/// `source → decode → convert → encode → mux → segment → output`
///
/// # Streaming Characteristics
/// - H.264 video encoding at 1000 kbps bitrate
/// - MPEG-TS container format for segments
/// - Automatic segment rotation for live-like streaming
/// - Compatible with HTML5 video players and iOS/Android devices
///
/// # Example
/// ```rust
/// let pipeline = create_hls_stream_pipeline(
///     "https://example.com/video.mp4",
///     "/output/stream"
/// );
/// // Creates: /output/stream/segment_00001.ts, segment_00002.ts, ..., playlist.m3u8
/// ```
pub fn create_hls_stream_pipeline(source_url: &str, output_dir: &str) -> String {
    // ---

    format!(
        "souphttpsrc location={source_url} ! decodebin ! videoconvert ! x264enc bitrate=1000 ! mpegtsmux ! hlssink location={output_dir}/segment_%05d.ts playlist-location={output_dir}/playlist.m3u8 max-files=10"
    )
}

#[cfg(test)]
mod tests {
    // ---

    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    /// Ensures GStreamer is initialized exactly once for all tests.
    ///
    /// GStreamer initialization is not thread-safe and should only be called once
    /// per process. This function uses std::sync::Once to guarantee single initialization
    /// even when tests run in parallel.
    fn ensure_gstreamer_init() {
        // ---
        INIT.call_once(|| {
            gstreamer::init().expect("Failed to initialize GStreamer for tests");
        });
    }

    #[test]
    fn test_validate_pipeline_string_valid_cases() {
        // ---
        ensure_gstreamer_init();

        // Use basic elements more likely to be available on GitLab runner
        assert!(validate_pipeline_string("fakesrc ! fakesink").is_ok());
        assert!(validate_pipeline_string("fakesrc ! identity ! fakesink").is_ok());
    }

    #[test]
    fn test_validate_pipeline_string_invalid_cases() {
        // ---
        ensure_gstreamer_init();

        // Empty string
        let result = validate_pipeline_string("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));

        // Whitespace only
        let result = validate_pipeline_string("   ");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));

        // No connections (missing !)
        let result = validate_pipeline_string("videotestsrc autovideosink");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("connection"));

        // Single element with no connection
        let result = validate_pipeline_string("videotestsrc");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("connection"));
    }

    #[test]
    fn test_create_conversion_pipeline_supported_formats() {
        // ---
        let source = "https://example.com/video.mp4";
        let output = "output.webm";

        // WebM format
        let result = create_conversion_pipeline(source, "webm", output);
        assert!(result.is_ok());
        let pipeline = result.unwrap();
        assert!(pipeline.contains("vp8enc"));
        assert!(pipeline.contains("webmmux"));

        // MP4 format
        let result = create_conversion_pipeline(source, "mp4", "output.mp4");
        assert!(result.is_ok());
        let pipeline = result.unwrap();
        assert!(pipeline.contains("x264enc"));
        assert!(pipeline.contains("mp4mux"));

        // AVI format
        let result = create_conversion_pipeline(source, "avi", "output.avi");
        assert!(result.is_ok());
        let pipeline = result.unwrap();
        assert!(pipeline.contains("x264enc"));
        assert!(pipeline.contains("avimux"));
    }

    #[test]
    fn test_create_conversion_pipeline_unsupported_format() {
        // ---
        let result = create_conversion_pipeline(
            "https://example.com/video.mp4",
            "unsupported",
            "output.xyz",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported output format"));
    }

    #[test]
    fn test_create_thumbnail_pipeline() {
        // ---
        let pipeline = create_thumbnail_pipeline(
            "https://example.com/video.mp4",
            "thumb.png",
            640,
            480,
            "00:01:30",
        );

        assert!(pipeline.contains("souphttpsrc"));
        assert!(pipeline.contains("decodebin"));
        assert!(pipeline.contains("videoconvert"));
        assert!(pipeline.contains("videoscale"));
        assert!(pipeline.contains("width=640"));
        assert!(pipeline.contains("height=480"));
        assert!(pipeline.contains("pngenc"));
        assert!(pipeline.contains("thumb.png"));
    }

    #[test]
    fn test_create_hls_stream_pipeline() {
        // ---
        let pipeline = create_hls_stream_pipeline("https://example.com/video.mp4", "/output/dir");

        assert!(pipeline.contains("souphttpsrc"));
        assert!(pipeline.contains("decodebin"));
        assert!(pipeline.contains("x264enc bitrate=1000"));
        assert!(pipeline.contains("mpegtsmux"));
        assert!(pipeline.contains("hlssink"));
        assert!(pipeline.contains("/output/dir/segment_%05d.ts"));
        assert!(pipeline.contains("/output/dir/playlist.m3u8"));
    }
}
