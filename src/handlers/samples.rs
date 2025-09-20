use axum::response::Json;

// Import through gateway
use crate::models::SampleMedia;

// Sample media URLs (those we discussed earlier)
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

pub async fn list_sample_media() -> Json<Vec<SampleMedia>> {
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

pub async fn health_check() -> Json<serde_json::Value> {
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
