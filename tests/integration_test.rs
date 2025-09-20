use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::{routing::get, Router};
use media_pipeline_service::{analyze_media, AppState}; // Specific imports instead of wildcard
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn test_analyze_endpoint_success() {
    // ---
    // Initialize GStreamer for the test
    gstreamer::init().unwrap();

    // Create test app state
    let app_state: AppState = Arc::new(Mutex::new(HashMap::new()));

    // Build the app (same as in main.rs)
    let app = Router::new()
        .route("/analyze/*url", get(analyze_media))
        .with_state(app_state);

    // Test URL - use a simple, reliable public domain video
    let test_url = "https://archive.org/download/night-15441/night-15441.mp4";
    let encoded_url = urlencoding::encode(test_url);

    // Create request
    let request = Request::builder()
        .uri(format!("/analyze/{}", encoded_url))
        .body(Body::empty())
        .unwrap();

    // Send request
    let response = app.oneshot(request).await.unwrap();

    // Verify response status
    assert_eq!(response.status(), StatusCode::OK);

    // Extract and verify response body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify required fields are present
    assert!(json.get("url").is_some());
    assert!(json.get("format").is_some());
    assert!(json.get("analysis_timestamp").is_some());

    // Verify URL was decoded correctly
    assert_eq!(json["url"].as_str().unwrap(), test_url);

    // Verify format is reasonable (not "unknown")
    let format = json["format"].as_str().unwrap();
    assert_ne!(format, "unknown");
    assert!(format.contains("video") || format.contains("mp4"));

    // Verify timestamp is valid ISO format
    let timestamp = json["analysis_timestamp"].as_str().unwrap();
    assert!(chrono::DateTime::parse_from_rfc3339(timestamp).is_ok());
}

#[tokio::test]
async fn test_analyze_endpoint_invalid_url() {
    // ---
    // Initialize GStreamer for the test
    gstreamer::init().unwrap();

    // Create test app state
    let app_state: AppState = Arc::new(Mutex::new(HashMap::new()));

    // Build the app
    let app = Router::new()
        .route("/analyze/*url", get(analyze_media))
        .with_state(app_state);

    // Test with invalid URL
    let invalid_url = "not-a-valid-url";
    let encoded_url = urlencoding::encode(invalid_url);

    // Create request
    let request = Request::builder()
        .uri(format!("/analyze/{}", encoded_url))
        .body(Body::empty())
        .unwrap();

    // Send request
    let response = app.oneshot(request).await.unwrap();

    // Should return error status
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Extract and verify error response
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify error structure
    assert!(json.get("error").is_some());
    assert_eq!(json["error"].as_str().unwrap(), "Failed to analyze media");
}

#[tokio::test]
async fn test_analyze_endpoint_url_decoding() {
    // ---
    // Test that URL decoding works correctly
    let test_url = "https://example.com/path with spaces/file.mp4";
    let encoded_url = urlencoding::encode(test_url);

    // This tests the decoding logic without hitting the network
    let decoded = urlencoding::decode(&encoded_url).unwrap();
    assert_eq!(decoded.as_ref(), test_url);
}
