use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Result;

/// Represents a prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Unique identifier for the prompt
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what the prompt does
    pub description: String,
    /// The actual prompt template
    pub template: String,
    /// Optional parameters for the template
    pub parameters: Option<Value>,
}

/// Prompt manager trait
#[async_trait]
pub trait PromptManager: Send + Sync {
    /// Lists available prompts
    async fn list_prompts(&self) -> Result<Vec<Prompt>>;

    /// Gets a specific prompt by ID
    async fn get_prompt(&self, id: &str) -> Result<Prompt>;

    /// Executes a prompt with given parameters
    async fn execute_prompt(&self, id: &str, params: Option<Value>) -> Result<Value>;
}
