//e gstreamer::prelude::*;

// Import sibling through super
use super::MediaInfo;

pub fn validate_pipeline_string(pipeline_string: &str) -> Result<(), String> {
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
        Err(e) => Err(format!("Invalid pipeline syntax: {}", e)),
    }
}

pub fn get_media_info(url: &str) -> anyhow::Result<MediaInfo> {
    // Create a discoverer pipeline to get media info
    let pipeline_string = format!("souphttpsrc location={} ! decodebin ! fakesink", url);

    let _pipeline = gstreamer::parse_launch(&pipeline_string)?;

    // This is a simplified version - in a real implementation,
    // you'd use GStreamer's discoverer API
    Ok(MediaInfo {
        duration: None,
        width: None,
        height: None,
        bitrate: None,
        format: "unknown".to_string(),
    })
}

// Utility function to create common pipeline patterns
pub fn create_conversion_pipeline(
    source_url: &str,
    output_format: &str,
    output_path: &str,
) -> Result<String, String> {
    match output_format {
        "webm" => Ok(format!(
            "souphttpsrc location={} ! decodebin ! videoconvert ! vp8enc ! webmmux ! filesink location={}",
            source_url, output_path
        )),
        "mp4" => Ok(format!(
            "souphttpsrc location={} ! decodebin ! videoconvert ! x264enc ! mp4mux ! filesink location={}",
            source_url, output_path
        )),
        "avi" => Ok(format!(
            "souphttpsrc location={} ! decodebin ! videoconvert ! x264enc ! avimux ! filesink location={}",
            source_url, output_path
        )),
        _ => Err(format!("Unsupported output format: {}", output_format)),
    }
}

pub fn create_thumbnail_pipeline(
    source_url: &str,
    output_path: &str,
    width: u32,
    height: u32,
    _timestamp: &str,
) -> String {
    format!(
        "souphttpsrc location={} ! decodebin ! videoconvert ! videoscale ! video/x-raw,width={},height={} ! pngenc ! filesink location={}",
        source_url, width, height, output_path
    )
}

pub fn create_hls_stream_pipeline(source_url: &str, output_dir: &str) -> String {
    format!(
        "souphttpsrc location={} ! decodebin ! videoconvert ! x264enc bitrate=1000 ! mpegtsmux ! hlssink location={}/segment_%05d.ts playlist-location={}/playlist.m3u8 max-files=10",
        source_url, output_dir, output_dir
    )
}
