use async_trait::async_trait;
use serde_json::Value;

use crate::Result;

/// Client configuration options
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Client name or identifier
    pub name: String,
    /// Client version
    pub version: String,
    /// Root directories for context
    pub roots: Vec<String>,
}

/// Represents an MCP client
#[async_trait]
pub trait Client: Send + Sync {
    /// Returns the client configuration
    fn config(&self) -> &ClientConfig;

    /// Handles a sampling request from the server
    async fn handle_sampling(&self, prompt: Value) -> Result<Value>;

    /// Provides context from root directories
    async fn get_root_context(&self, path: &str) -> Result<Value>;
}

pub mod roots;
pub mod sampling;
