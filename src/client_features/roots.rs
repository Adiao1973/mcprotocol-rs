use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Result;

/// Represents a root directory for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Root {
    /// Path to the root directory
    pub path: String,
    /// Optional name for the root
    pub name: Option<String>,
    /// Optional pattern for files to include
    pub include_pattern: Option<String>,
    /// Optional pattern for files to exclude
    pub exclude_pattern: Option<String>,
}

/// Root directory manager trait
#[async_trait]
pub trait RootManager: Send + Sync {
    /// Lists all registered roots
    fn list_roots(&self) -> Vec<Root>;

    /// Adds a new root directory
    fn add_root(&mut self, root: Root) -> Result<()>;

    /// Removes a root directory
    fn remove_root(&mut self, path: &str) -> Result<()>;

    /// Gets context from a specific path within roots
    async fn get_context(&self, path: &str) -> Result<Value>;
}
