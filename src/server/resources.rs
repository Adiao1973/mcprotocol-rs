use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Result;

/// Represents a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Unique identifier for the resource
    pub id: String,
    /// Resource type (e.g., "file", "git", "database")
    pub type_: String,
    /// Resource metadata
    pub metadata: Value,
    /// Optional content
    pub content: Option<Value>,
}

/// Resource manager trait
#[async_trait]
pub trait ResourceManager: Send + Sync {
    /// Lists available resources
    async fn list_resources(&self) -> Result<Vec<Resource>>;

    /// Gets a specific resource by ID
    async fn get_resource(&self, id: &str) -> Result<Resource>;

    /// Creates a new resource
    async fn create_resource(&self, resource: Resource) -> Result<()>;

    /// Updates an existing resource
    async fn update_resource(&self, id: &str, resource: Resource) -> Result<()>;

    /// Deletes a resource
    async fn delete_resource(&self, id: &str) -> Result<()>;
}
