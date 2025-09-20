use gstreamer::prelude::*;
use std::time::Duration;

// Import from parent module
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
        Err(e) => Err(format!("Invalid pipeline syntax: {e}")),
    }
}

pub fn get_media_info(url: &str) -> anyhow::Result<MediaInfo> {
    use gstreamer::MessageView;

    // Create a discovery pipeline - we'll probe the media without fully decoding
    let pipeline_string = format!(
        "souphttpsrc location={} ! typefind ! identity signal-handoffs=false ! fakesink sync=false",
        url
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
        for pad_result in pipeline.iterate_pads() {
            if let Ok(pad) = pad_result {
                if let Some(caps) = pad.current_caps() {
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
