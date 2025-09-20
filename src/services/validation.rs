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

#[cfg(test)]
mod tests {
    // ---

    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn ensure_gstreamer_init() {
        INIT.call_once(|| {
            gstreamer::init().expect("Failed to initialize GStreamer for tests");
        });
    }

    #[test]
    fn test_validate_pipeline_string_valid_cases() {
        // ---
        ensure_gstreamer_init();

        // Basic valid pipeline
        assert!(validate_pipeline_string("videotestsrc ! autovideosink").is_ok());

        // Multiple elements
        assert!(validate_pipeline_string("videotestsrc ! videoconvert ! autovideosink").is_ok());

        // With properties
        assert!(validate_pipeline_string("videotestsrc pattern=ball ! autovideosink").is_ok());

        // Audio pipeline
        assert!(validate_pipeline_string("audiotestsrc ! autoaudiosink").is_ok());
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
