//! HTTP request handlers for the Media Pipeline Service API.
//!
//! This module serves as the gateway for all HTTP endpoint handlers, implementing
//! the Explicit Module Boundary Pattern (EMBP) to control the public handler API
//! and maintain clean separation between different categories of endpoints.
//!
//! # Handler Organization
//!
//! Handlers are logically grouped into three categories based on functionality:
//! - **Media Processing**   : Core media operations (conversion, thumbnails, streaming)
//! - **Pipeline Management**: CRUD operations for custom pipeline lifecycles
//! - **Service Operations** : Health checks, samples, and service discovery
//!
//! # EMBP Implementation
//!
//! This gateway module defines the complete public API for HTTP handlers,
//! ensuring that only validated, stable functions are exposed to the routing
//! layer. Internal implementation details remain private within sub-modules.
//!
//! # Request/Response Architecture
//!
//! All handlers follow consistent patterns:
//! - Accept well-defined request DTOs via JSON deserialization
//! - Return standardized response structures or error types
//! - Use structured logging for operational observability
//! - Implement comprehensive input validation and error handling
//!
//! # State Management
//!
//! Handlers share application state through the `AppState` type alias,
//! providing thread-safe access to pipeline tracking and service configuration.
//! This enables coordinated management of pipeline lifecycles across endpoints.

// ---

// EMBP Handlers Gateway: Controls public API for all handler functions
mod media;
mod pipeline;
mod samples;

// ---

// Public exports - this defines the entire public handlers API
pub use media::{convert_media, create_stream, generate_thumbnail};
pub use pipeline::{analyze_media, create_pipeline, get_pipeline, list_pipelines, stop_pipeline};
pub use samples::{health_check, list_sample_media};

// Import stuff needed to define AppState below
use crate::models::PipelineInfo;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Shared application state for pipeline tracking across all handlers.
///
/// Provides thread-safe access to the pipeline registry, enabling
/// coordinated management of pipeline lifecycles across all HTTP endpoints.
pub type AppState = Arc<Mutex<HashMap<String, PipelineInfo>>>;
