# Media Pipeline Service

A REST API service for media processing using GStreamer and Rust, demonstrating modern Rust architecture patterns and professional development practices.

## Features

- **Media Format Conversion** - Convert videos between WebM, MP4, and AVI formats
- **Thumbnail Generation** - Extract thumbnails from video content at specified timestamps
- **HLS Streaming** - Create HTTP Live Streaming pipelines for real-time video delivery
- **Pipeline Management** - Create, monitor, and control custom GStreamer pipelines
- **Media Analysis** - Analyze media files to extract format, resolution, and metadata
- **Built-in Samples** - Pre-configured sample media for testing and demonstration
- **Professional CLI** - Command-line interface with smart colorization and flexible configuration
- **Integration Testing** - Comprehensive HTTP test suite with clean output management

## Architecture

This project implements the **Explicit Module Boundary Pattern (EMBP)** for clean, maintainable Rust code. This pattern aligns with interface design principles discussed in Jon Gjengset's _Rust for Rustaceans_ (Chapter 3: _Designing Interfaces_), particularly around controlling API boundaries and maintaining clear separation of concerns. For a detailed explanation of EMBP principles and implementation, see the [EMBP documentation](https://github.com/JohnBasrai/architecture-patterns/blob/main/rust/embp.md).

```
src/
├── handlers/          # HTTP endpoint handlers
│   ├── mod.rs         #   Gateway controlling public handler API
│   ├── media.rs       #   Media processing endpoints
│   ├── pipeline.rs    #   Pipeline CRUD operations
│   └── samples.rs     #   Sample data and health checks
├── models/            # Data structures and DTOs
│   ├── mod.rs         #   Gateway controlling public model API
│   ├── pipeline.rs    #   Pipeline state management
│   ├── requests.rs    #   Request DTOs
│   └── responses.rs   #   Response DTOs
├── services/          # Business logic and GStreamer integration
│   ├── mod.rs         #   Gateway controlling public service API
│   └── validation.rs  #   Pipeline validation and utilities
├── main.rs            #   Application entry point and routing
└── tests/             # Integration test suite
    └── integration_test.rs  # HTTP API testing
```

## Prerequisites

- **Rust** (edition 2021 or later)
- **GStreamer** development libraries
  - Ubuntu/Debian: `sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev`
  - macOS: `brew install gstreamer gst-plugins-base`
  - Windows: Install GStreamer from [gstreamer.freedesktop.org](https://gstreamer.freedesktop.org)

## Quick Start

1. **Clone and build:**
   ```bash
   git clone <repository-url>
   cd media-pipeline-service
   cargo build
   ```

2. **Run the service:**
   ```bash
   # Run with default settings (localhost:8080)
   cargo run
   
   # Run on custom port
   cargo run -- --port 3000
   
   # Run on specific host and port
   cargo run -- --host 127.0.0.1 --port 8081
   
   # Control log colorization
   cargo run -- --color always    # Force colors
   cargo run -- --color never     # No colors
   cargo run -- --color auto      # Auto-detect (default)
   
   # See all CLI options
   cargo run -- --help
   ```

3. **Test the service:**
   ```bash
   # Check health
   curl http://localhost:8080/health
   
   # Run all tests
   cargo test
   ```

## CLI Options

The service includes a professional command-line interface:

```bash
Usage: media-pipeline-service [OPTIONS]

Options:
  -p, --port <PORT>        Port to bind the server to [default: 8080]
      --host <HOST>        Host address to bind the server to [default: 0.0.0.0]
      --color <WHEN>       Coloring [default: auto] [possible values: auto, always, never]
  -h, --help               Print help
  -V, --version            Print version
```

**Smart colorization:**
- `auto` - Colors when output is to terminal, plain text when redirected
- `always` - Force colored output regardless of destination
- `never` - Disable colored output completely

## API Endpoints

### Health and Information
- `GET /health` - Service health check and GStreamer version info
- `GET /samples` - List available sample media for testing

### Media Processing
- `POST /convert` - Convert media between formats
- `POST /thumbnail` - Generate thumbnail from video
- `POST /stream` - Create HLS streaming pipeline
- `GET /analyze/{url}` - Analyze media file metadata

### Pipeline Management
- `GET /pipelines` - List all active pipelines
- `POST /pipelines` - Create custom GStreamer pipeline
- `GET /pipelines/{id}` - Get specific pipeline status
- `DELETE /pipelines/{id}` - Stop pipeline execution

## Usage Examples

### Convert Video Format
```bash
curl -X POST http://localhost:8080/convert \
  -H "Content-Type: application/json" \
  -d '{
    "source_url": "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
    "output_format": "webm"
  }'
```

### Generate Thumbnail
```bash
curl -X POST http://localhost:8080/thumbnail \
  -H "Content-Type: application/json" \
  -d '{
    "source_url": "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
    "timestamp": "00:01:30",
    "width": 640,
    "height": 480
  }'
```

### Create HLS Stream
```bash
curl -X POST http://localhost:8080/stream \
  -H "Content-Type: application/json" \
  -d '{
    "source_url": "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
    "stream_type": "hls"
  }'
```

### Analyze Media File
```bash
# URL-encode the media URL for the path parameter
curl http://localhost:8080/analyze/https%3A//commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4
```

### Create Custom Pipeline
```bash
curl -X POST http://localhost:8080/pipelines \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Audio extraction pipeline",
    "pipeline": "souphttpsrc location=https://example.com/video.mp4 ! decodebin ! audioconvert ! vorbisenc ! oggmux ! filesink location=output.ogg"
  }'
```

### List Sample Media
```bash
curl http://localhost:8080/samples
```

## Testing

The project includes comprehensive testing that verifies functionality at multiple levels:

```bash
# Run all tests (unit + integration)
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test

# Run specific test with output
cargo test test_analyze_endpoint_integration -- --nocapture

# Run tests sequentially (cleaner output)
cargo test -- --test-threads=1
```

**Test Features:**
- **Unit tests** for pipeline validation and construction functions
- **Integration tests** for end-to-end HTTP functionality
- Automatic server lifecycle management
- Clean output with server log capture
- DRY test infrastructure with helper functions
- Cross-platform compatibility
- Error scenario coverage

### CI/CD Pipeline

The project includes automated testing via GitHub Actions:

- **Automated quality checks** on every push and pull request
- **Cross-platform testing** on Ubuntu with GStreamer dependencies
- **Code formatting** verification with `cargo fmt --check`
- **Linting** with `cargo clippy` treating warnings as errors
- **Unit tests** for core functionality validation
- **Release builds** to verify production readiness

*Note: Integration tests are currently disabled in CI due to hanging issues on GitHub Actions runners, but can be run locally with `cargo test --test integration_test`.*

## Local Quality Checks

Run the same checks as CI locally:

```bash
# Full quality check pipeline
cargo fmt --check && cargo clippy -- -D warnings && cargo test

# Individual checks
cargo fmt --check     # Verify formatting
cargo clippy         # Check for lints
cargo test          # Run all tests
cargo build --release  # Verify release build
```

## GStreamer Integration

The service demonstrates several key GStreamer concepts:

**Pipeline Validation**: All pipeline strings are validated before execution using `gstreamer::parse_launch()`.

**Modular Pipeline Construction**: Common pipeline patterns are built programmatically for different use cases (conversion, thumbnails, streaming).

**Error Handling**: GStreamer errors are properly captured and returned as structured API responses.

**State Management**: Pipeline states (Created, Playing, Paused, Stopped, Error) are tracked and exposed through the API.

**Media Discovery**: Real media analysis using GStreamer's discovery capabilities with proper timeout and error handling.

## Sample Pipelines

The service includes pre-built pipeline generators for common operations:

**Video Conversion:**
```
souphttpsrc location={url} ! decodebin ! videoconvert ! x264enc ! mp4mux ! filesink location={output}
```

**Thumbnail Generation:**
```
souphttpsrc location={url} ! decodebin ! videoconvert ! videoscale ! video/x-raw,width={w},height={h} ! pngenc ! filesink location={output}
```

**HLS Streaming:**
```
souphttpsrc location={url} ! decodebin ! videoconvert ! x264enc bitrate=1000 ! mpegtsmux ! hlssink location={dir}/segment_%05d.ts playlist-location={dir}/playlist.m3u8
```

## Error Handling

The API provides structured error responses with detailed information:

```json
{
  "error": "Invalid pipeline configuration",
  "details": "Pipeline must contain at least one element connection (!)"
}
```

Common error scenarios include:
- Invalid GStreamer pipeline syntax
- Unsupported media formats
- Network accessibility issues with source URLs
- Missing GStreamer plugins

## Development

**Run with logging:**
```bash
RUST_LOG=info cargo run

# Run on custom port with logging
RUST_LOG=debug cargo run -- --port 3000 --color always
```

**Development workflow:**
```bash
# Check code quality
cargo clippy

# Format code
cargo fmt

# Run tests with output
cargo test -- --nocapture

# Clean builds
cargo clean && cargo build
```

**CI/CD Ready:**
- Integration tests suitable for automated pipelines
- Clean test output management
- Cross-platform compatibility
- Proper process cleanup

## Technology Stack

- **Rust** - Systems programming language for performance and safety
- **Axum** - Modern async web framework for Rust
- **GStreamer** - Multimedia framework for pipeline-based media processing
- **Tokio** - Async runtime for concurrent request handling
- **Serde** - Serialization framework for JSON API responses
- **Tracing** - Structured logging and observability
- **Clap** - Command-line argument parsing with derive macros
- **Reqwest** - HTTP client for integration testing

## Professional Features

- **EMBP Architecture** - Clean module boundaries and explicit APIs
- **Signal Handling** - Graceful shutdown with Ctrl-C support
- **Smart CLI** - Terminal-aware colorization and flexible configuration
- **Comprehensive Testing** - Professional test suite with unit and integration tests
- **CI/CD Pipeline** - Automated quality checks and cross-platform testing
- **Error Management** - Comprehensive error handling and reporting
- **Documentation** - Extensive inline documentation and examples

## Future Enhancements

- WebSocket support for real-time pipeline status updates
- File upload endpoints for local media processing
- Advanced audio processing pipelines
- Integration with cloud storage services
- Docker containerization with GStreamer dependencies
- Prometheus metrics for pipeline performance monitoring

## License

This project is licensed under the MIT License - see the LICENSE file for details.

---

**Note**: While this service demonstrates GStreamer and Rust integration patterns, it would require additional security, authentication, and resource management features for production deployment.
