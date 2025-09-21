//! Integration tests for the Media Pipeline Service REST API
//!
//! These tests spawn actual server processes and make HTTP requests to verify
//! end-to-end functionality. Each test runs on a separate port to avoid conflicts.
//!
//! ## Test Coverage
//! - `/health` - Server health and GStreamer version info
//! - `/samples` - Sample media listing functionality  
//! - `/analyze/{url}` - Media analysis endpoint (success and error cases)
//!
//! ## Test Infrastructure
//! - Uses reqwest for HTTP client functionality
//! - Spawns server processes via `cargo run` with unique ports
//! - Automatic server lifecycle management (startup, wait, cleanup)
//! - DRY helpers for URL construction and server management
//!
//! ## Running Tests
//! ```bash
//! cargo test --test integration_test           # Run all integration tests
//! cargo test test_health_endpoint              # Run specific test
//! cargo test -- --nocapture                   # Show server output
//! ```
//!
//! ## Cleanup
//! If tests are interrupted and leave orphaned processes:
//! ```bash
//! pkill -f "media-pipeline-service.*--port 808[1-9]"
//! ```

use serde_json::Value;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::sleep;

// Port allocation for tests
static NEXT_PORT: AtomicU16 = AtomicU16::new(8081);

fn get_test_port() -> u16 {
    NEXT_PORT.fetch_add(1, Ordering::SeqCst)
}

// Macro for URL construction
macro_rules! endpoint_url {
    ($base:expr, $path:expr) => {
        format!("{}/{}", $base, $path)
    };
    ($base:expr, $path:expr, $param:expr) => {
        format!("{}/{}/{}", $base, $path, $param)
    };
}

// Helper struct to manage test server lifecycle
struct TestServer {
    process: Child,
    client: reqwest::Client,
    base_url: String,
    port: u16,
}

impl TestServer {
    // ---

    async fn start() -> Self {
        // ---

        let port = get_test_port();
        let base_url = format!("http://localhost:{port}");

        // Use pre-built binary for faster, cleaner testing
        let process = Command::new("cargo")
            .args([
                "run",
                "--",
                "--host",
                "localhost",
                "--port",
                &port.to_string(),
                "--color",
                "never",
            ])
            .stdout(std::process::Stdio::piped()) // Capture for debugging
            .stderr(std::process::Stdio::piped()) // Capture for debugging
            .spawn()
            .expect("Failed to start server - ensure 'cargo build' has been run");

        let pid = process.id().expect("Failed to get process ID");

        // Wait for server to start
        let client = reqwest::Client::new();
        let timeout = Duration::from_secs(10);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if let Ok(response) = client.get(endpoint_url!(base_url, "health")).send().await {
                if response.status().is_success() {
                    println!("✓ Test server started: PID {pid} on port {port}");
                    return TestServer {
                        process,
                        client,
                        base_url,
                        port,
                    };
                }
            }
            sleep(Duration::from_millis(100)).await;
        }

        // If startup failed, dump server output for debugging
        if let Ok(output) = process.wait_with_output().await {
            eprintln!("❌ Server startup failed on port {port}:");
            eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        }

        panic!("Server failed to start within 10 seconds on port {port}");
    }

    async fn shutdown(mut self) {
        if let Err(e) = self.process.kill().await {
            eprintln!(
                "Warning: Failed to kill server on port {}: {}",
                self.port, e
            );
        }
    }
} // impl TestServer

#[tokio::test]
async fn test_pipeline_lifecycle_integration() {
    // ---

    let server = TestServer::start().await;

    // Create a custom pipeline
    let create_request = serde_json::json!({
        "description": "Test audio extraction pipeline",
        "pipeline": "fakesrc ! fakesink"
    });

    let create_response = server
        .client
        .post(&format!("{}/pipelines", server.base_url))
        .header("Content-Type", "application/json")
        .json(&create_request)
        .send()
        .await
        .expect("Failed to create pipeline");

    assert_eq!(create_response.status(), 200);

    let pipeline_data: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");

    let pipeline_id = pipeline_data["id"].as_str().unwrap();

    // Get specific pipeline
    let get_response = server
        .client
        .get(&format!("{}/pipelines/{}", server.base_url, pipeline_id))
        .send()
        .await
        .expect("Failed to get pipeline");

    assert_eq!(get_response.status(), 200);

    // List all pipelines and verify ours is there
    let list_response = server
        .client
        .get(&format!("{}/pipelines", server.base_url))
        .send()
        .await
        .expect("Failed to list pipelines");

    assert_eq!(list_response.status(), 200);

    let pipelines: serde_json::Value = list_response
        .json()
        .await
        .expect("Failed to parse list response");

    assert!(pipelines
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == pipeline_id));

    // Stop the pipeline
    let delete_response = server
        .client
        .delete(&format!("{}/pipelines/{}", server.base_url, pipeline_id))
        .send()
        .await
        .expect("Failed to stop pipeline");

    assert_eq!(delete_response.status(), 200);

    // Verify pipeline is now stopped
    let final_get_response = server
        .client
        .get(&format!("{}/pipelines/{}", server.base_url, pipeline_id))
        .send()
        .await
        .expect("Failed to get stopped pipeline");

    let stopped_pipeline: serde_json::Value = final_get_response
        .json()
        .await
        .expect("Failed to parse stopped pipeline");

    assert_eq!(stopped_pipeline["state"], "Stopped");
}

#[tokio::test]
async fn test_convert_media_integration() {
    // ---

    let server = TestServer::start().await;

    let convert_request = serde_json::json!({
        "source_url": "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
        "output_format": "webm"
    });

    let response = server
        .client
        .post(&format!("{}/convert", server.base_url))
        .header("Content-Type", "application/json")
        .json(&convert_request)
        .send()
        .await
        .expect("Failed to send convert request");

    assert_eq!(response.status(), 200);

    let convert_response: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse convert response");

    // Verify response structure
    assert!(convert_response["pipeline_id"].is_string());
    assert_eq!(convert_response["status"], "created");
    assert!(convert_response["message"]
        .as_str()
        .unwrap()
        .contains("webm"));
    assert!(convert_response["estimated_duration"].is_string());

    // Verify the pipeline was created and can be retrieved
    let pipeline_id = convert_response["pipeline_id"].as_str().unwrap();
    let pipeline_response = server
        .client
        .get(&format!("{}/pipelines/{}", server.base_url, pipeline_id))
        .send()
        .await
        .expect("Failed to get pipeline");

    assert_eq!(pipeline_response.status(), 200);

    let pipeline_data: serde_json::Value = pipeline_response
        .json()
        .await
        .expect("Failed to parse pipeline response");

    assert_eq!(pipeline_data["id"], pipeline_id);
    assert_eq!(pipeline_data["state"], "Created");
    assert!(pipeline_data["description"]
        .as_str()
        .unwrap()
        .contains("webm"));
}

#[tokio::test]
async fn test_analyze_endpoint_integration() {
    // ---
    let server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Test successful analysis
    let test_url =
        "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";
    let encoded_url = urlencoding::encode(test_url);

    let response = client
        .get(endpoint_url!(server.base_url, "analyze", encoded_url))
        .send()
        .await
        .expect("Failed to send request");

    // Verify response status
    assert_eq!(response.status(), 200);

    // Verify response content
    let json: Value = response.json().await.expect("Failed to parse JSON");

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

    // ---
    // Test error case - invalid URL
    let invalid_response = client
        .get(endpoint_url!(server.base_url, "analyze", "not-a-valid-url"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(invalid_response.status(), 422);

    let error_json: Value = invalid_response
        .json()
        .await
        .expect("Failed to parse error JSON");
    assert!(error_json.get("error").is_some());
    assert_eq!(
        error_json["error"].as_str().unwrap(),
        "Failed to analyze media"
    );

    // ---
    server.shutdown().await;
}

#[tokio::test]
async fn test_health_endpoint() {
    // ---
    let fname = "test_health_endpoint";

    println!("{fname}: Calling TestServer::start().await");
    let server = TestServer::start().await;

    println!("{fname}: Calling reqwest::Client::new()");
    let client = reqwest::Client::new();
    println!("{fname}: Doing client.send");

    // Test health endpoint
    let response = client
        .get(endpoint_url!(server.base_url, "health"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let json: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(json["status"].as_str().unwrap(), "healthy");
    assert!(json.get("gstreamer_version").is_some());
    assert!(json.get("endpoints").is_some());

    // ---
    server.shutdown().await;
}

#[tokio::test]
async fn test_samples_endpoint() {
    // ---
    let server = TestServer::start().await;
    let client = reqwest::Client::new();

    let response = client
        .get(endpoint_url!(server.base_url, "samples"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let json: Value = response.json().await.expect("Failed to parse JSON");
    let samples = json.as_array().expect("Expected array of samples");

    // Should have at least one sample
    assert!(!samples.is_empty());

    // Each sample should have required fields
    for sample in samples {
        assert!(sample.get("name").is_some());
        assert!(sample.get("url").is_some());
        assert!(sample.get("media_type").is_some());
        assert!(sample.get("description").is_some());
    }

    // ---
    server.shutdown().await;
}
