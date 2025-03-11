use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Result;

/// Represents a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Unique identifier for the tool
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what the tool does
    pub description: String,
    /// Tool parameters schema
    pub parameters: Value,
    /// Whether the tool requires user approval
    pub requires_approval: bool,
}

/// Tool manager trait
#[async_trait]
pub trait ToolManager: Send + Sync {
    /// Lists available tools
    async fn list_tools(&self) -> Result<Vec<Tool>>;

    /// Gets a specific tool by ID
    async fn get_tool(&self, id: &str) -> Result<Tool>;

    /// Executes a tool with given parameters
    async fn execute_tool(&self, id: &str, params: Value) -> Result<Value>;

    /// Cancels a running tool execution
    async fn cancel_tool(&self, id: &str) -> Result<()>;
}
