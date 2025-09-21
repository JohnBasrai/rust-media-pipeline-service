//! Pipeline state management and information structures.
//!
//! This module defines the core data structures for representing GStreamer pipeline
//! instances and their execution state throughout their lifecycle. It provides
//! comprehensive tracking of pipeline metadata, execution status, and creation context.
//!
//! # Pipeline Lifecycle
//!
//! Pipelines progress through defined states from creation to completion:
//! `Created → Playing → (Paused) → Stopped/Error`
//!
//! # State Management
//!
//! - **Created**: Pipeline validated and queued for execution
//! - **Playing**: Pipeline actively processing media
//! - **Paused**: Pipeline temporarily suspended (resumable)
//! - **Stopped**: Pipeline completed or manually terminated
//! - **Error**: Pipeline failed with diagnostic information

use serde::{Deserialize, Serialize};

/// Comprehensive information about a GStreamer pipeline instance.
///
/// Represents a complete pipeline with its metadata, current state, and execution
/// context. This structure is used both for tracking active pipelines and for
/// API responses that return pipeline information to clients.
///
/// # Example JSON Representation
/// ```json
/// {
///   "id": "550e8400-e29b-41d4-a716-446655440000",
///   "description": "Convert to webm",
///   "state": "Created",
///   "pipeline_string": "souphttpsrc location=... ! decodebin ! ...",
///   "created_at": "2024-09-21T10:30:00Z",
///   "source_url": "https://example.com/video.mp4"
/// }
/// ```
///
/// # Usage Patterns
/// - Stored in the application state for tracking active pipelines
/// - Returned by pipeline status and listing endpoints
/// - Updated as pipeline state changes during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineInfo {
    // ---
    /// Unique identifier for this pipeline instance (UUID v4 format)
    pub id: String,

    /// Human-readable description of what this pipeline does
    pub description: String,

    /// Current execution state of the pipeline
    pub state: PipelineState,

    /// Complete GStreamer pipeline string used for execution
    pub pipeline_string: String,

    /// ISO 8601 timestamp when the pipeline was created
    pub created_at: String,

    /// Optional source URL if this pipeline processes remote media
    /// None for pipelines that don't use network sources
    pub source_url: Option<String>,
}

/// Enumeration of all possible pipeline execution states.
///
/// Represents the current status of a GStreamer pipeline throughout its lifecycle.
/// States generally progress linearly, though transitions between Playing and Paused
/// can occur multiple times, and any state can transition to Error.
///
/// # State Transitions
/// ```text
/// Created → Playing → Stopped (normal completion)
///    ↓         ↕         ↑
/// Error ← Paused ←──────┘
/// ```
///
/// # JSON Serialization
/// States serialize as simple strings in JSON, with Error states including
/// the error message as additional context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineState {
    // ---
    /// Pipeline has been created and validated but not yet started.
    ///
    /// This is the initial state after successful pipeline creation and validation.
    /// The pipeline is ready to begin execution but has not been submitted to
    /// the GStreamer runtime yet.
    Created,

    /// Pipeline is actively processing media data.
    ///
    /// The GStreamer pipeline is running and actively processing the media stream.
    /// This state indicates normal operation with data flowing through the pipeline
    /// elements.
    Playing,

    /// Pipeline execution is temporarily suspended.
    ///
    /// The pipeline has been paused and can be resumed. This state preserves
    /// the current position in the media stream and allows for resumption
    /// without restarting from the beginning.
    Paused,

    /// Pipeline has completed execution or been manually terminated.
    ///
    /// This is a terminal state indicating either successful completion of
    /// processing or manual termination by user request. Stopped pipelines
    /// cannot be resumed.
    Stopped,

    /// Pipeline encountered an error and cannot continue.
    ///
    /// Terminal error state with diagnostic information. The String contains
    /// details about what went wrong, typically including GStreamer error
    /// messages or validation failures.
    ///
    /// # Error Examples
    /// - "GStreamer error: Could not link elements"
    /// - "Source URL not accessible: HTTP 404"
    /// - "Invalid pipeline syntax: unknown element 'badelem'"
    Error(String),
}
