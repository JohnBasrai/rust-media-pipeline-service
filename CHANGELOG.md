# Changelog

All notable changes to the Media Pipeline Service will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Smart CLI colorization** with `--color auto/always/never` option using terminal detection
- **Professional HTTP integration tests** with comprehensive test coverage
- **DRY test infrastructure** with automatic server lifecycle management
- **File-level documentation** for integration test suite

### Changed
- **Improved test output management** - server logs captured and shown only on failure
- **Streamlined codebase** by removing unused code and consolidating MediaInfo struct
- **Enhanced CLI experience** with intelligent color detection for terminals vs pipes

### Removed
- Unused `PipelineService` struct and associated methods
- Unused `quality` field from `ConvertRequest`
- Unused `bitrate` field from `StreamRequest`
- Hybrid library structure (`lib.rs`) - now pure binary application

### Technical Improvements
- Integration tests use reqwest for true HTTP testing
- Automatic port allocation prevents test conflicts
- Clean test output suitable for CI/CD pipelines
- Proper process cleanup and error handling

## [0.1.0] - 2025-01-XX

### Added
- Initial release of Media Pipeline Service
- REST API built with Axum web framework
- GStreamer integration for media processing
- Explicit Module Boundary Pattern (EMBP) architecture implementation

#### Core Features
- **Media format conversion** between WebM, MP4, and AVI formats
- **Thumbnail generation** from video content with customizable dimensions and timestamps
- **HLS streaming pipeline creation** for HTTP Live Streaming
- **Custom GStreamer pipeline support** with validation
- **Media file analysis** endpoint for extracting metadata and format information

#### API Endpoints
- `GET /health` - Service health check with GStreamer version information
- `GET /samples` - List of sample media files for testing
- `POST /convert` - Convert media between supported formats
- `POST /thumbnail` - Generate thumbnails from video sources
- `POST /stream` - Create HLS streaming pipelines
- `GET /analyze/{url}` - Analyze media file metadata
- `GET /pipelines` - List all active pipelines
- `POST /pipelines` - Create custom GStreamer pipelines
- `GET /pipelines/{id}` - Get pipeline status and information
- `DELETE /pipelines/{id}` - Stop running pipelines

#### Architecture
- **EMBP Module Organization**: Clean separation of concerns with gateway modules
- **Pipeline Validation**: Pre-execution validation of GStreamer pipeline strings
- **State Management**: Comprehensive pipeline state tracking (Created, Playing, Paused, Stopped, Error)
- **Error Handling**: Structured error responses with detailed GStreamer error information
- **Modular Pipeline Construction**: Programmatic pipeline building for common use cases

#### CLI Features
- **Command-line interface** with clap derive for argument parsing
- **Configurable host and port** with sensible defaults
- **Graceful shutdown** handling with Ctrl-C signal capture
- **Smart colorization** with terminal detection

#### Dependencies
- **Core Framework**: Axum 0.7 for async HTTP handling
- **Media Processing**: GStreamer 0.21 with app and video plugins
- **CLI**: Clap 4.0 with derive features for argument parsing
- **Serialization**: Serde 1.0 for JSON API responses
- **Async Runtime**: Tokio 1.0 with full feature set
- **Logging**: Tracing 0.1 with tracing-subscriber 0.3
- **Testing**: Reqwest 0.11 for HTTP integration tests
- **Utilities**: anyhow, chrono, uuid, urlencoding, is-terminal

#### Sample Media Integration
- Big Buck Bunny (Blender Foundation short film)
- Elephant's Dream (Open source animation)
- Classical music samples
- Nature documentary footage
- All samples use publicly available, legally distributable content

#### Development Features
- Comprehensive structured logging with tracing
- Pipeline string validation before execution
- URL encoding support for media analysis endpoints
- Detailed API documentation with curl examples
- Clean error propagation from GStreamer to HTTP responses
- **Integration test suite** with automatic server management
- **Cross-platform compatibility** avoiding binary path dependencies

### Technical Highlights
- **Memory Safety**: Leverages Rust's ownership system for safe GStreamer integration
- **Concurrent Processing**: Async/await pattern for handling multiple pipeline operations
- **Type Safety**: Strong typing throughout the API with proper error handling
- **Modular Design**: EMBP pattern enables clean refactoring and maintainability
- **Professional Testing**: HTTP integration tests verify end-to-end functionality
- **CI/CD Ready**: Clean test output and reliable process management

### Known Limitations
- Pipeline execution is tracked but not actively managed (pipelines run to completion)
- File output locations are hardcoded relative paths
- No authentication or rate limiting implemented
- Limited to HTTP/HTTPS source URLs (no local file support)
- Basic media info discovery (full GStreamer discoverer API not yet integrated)

### Future Considerations
- Real-time pipeline status updates via WebSockets
- File upload support for local media processing
- Advanced audio processing pipelines
- Cloud storage integration
- Docker containerization
- Production security features