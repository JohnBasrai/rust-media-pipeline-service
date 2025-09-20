use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineInfo {
    pub id: String,
    pub description: String,
    pub state: PipelineState,
    pub pipeline_string: String,
    pub created_at: String,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineState {
    Created,
    Playing,
    Paused,
    Stopped,
    Error(String),
}
