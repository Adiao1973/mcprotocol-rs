use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Result;

/// Represents a sampling request from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRequest {
    /// The prompt to be processed
    pub prompt: Value,
    /// Optional sampling parameters
    pub parameters: Option<Value>,
    /// Optional stop sequences
    pub stop: Option<Vec<String>>,
}

/// Represents a sampling response to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingResponse {
    /// The generated text
    pub text: String,
    /// Optional metadata about the sampling
    pub metadata: Option<Value>,
}

/// Sampling handler trait
#[async_trait]
pub trait SamplingHandler: Send + Sync {
    /// Handles a sampling request
    async fn handle_request(&self, request: SamplingRequest) -> Result<SamplingResponse>;

    /// Cancels an ongoing sampling operation
    async fn cancel(&self) -> Result<()>;
}
