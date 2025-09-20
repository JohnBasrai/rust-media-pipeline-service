# Changelog

All notable changes to the Media Pipeline Service will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

#### Dependencies
- **Core Framework**: Axum 0.7 for async HTTP handling
- **Media Processing**: GStreamer 0.21 with app and video plugins
- **Serialization**: Serde 1.0 for JSON API responses
- **Async Runtime**: Tokio 1.0 with full feature set
- **Logging**: Tracing 0.1 with tracing-subscriber 0.3
- **Utilities**: anyhow, chrono, uuid, urlencoding

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

### Technical Highlights
- **Memory Safety**: Leverages Rust's ownership system for safe GStreamer integration
- **Concurrent Processing**: Async/await pattern for handling multiple pipeline operations
- **Type Safety**: Strong typing throughout the API with proper error handling
- **Modular Design**: EMBP pattern enables clean refactoring and maintainability

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