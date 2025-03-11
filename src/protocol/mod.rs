pub mod message;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use message::*;

/// Current protocol version
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// Represents a unique identifier for JSON-RPC requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientCapabilities {
    /// Root directory capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootCapability>,
    /// Sampling capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<Value>,
    /// Experimental features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Value>,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCapabilities {
    /// Prompt capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<FeatureCapability>,
    /// Resource capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceCapability>,
    /// Tool capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<FeatureCapability>,
    /// Logging capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Value>,
    /// Experimental features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Value>,
}

/// Root directory capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCapability {
    /// Support for list change notifications
    #[serde(default)]
    pub list_changed: bool,
}

/// Resource capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapability {
    /// Support for subscribing to changes
    #[serde(default)]
    pub subscribe: bool,
    /// Support for list change notifications
    #[serde(default)]
    pub list_changed: bool,
}

/// Feature capability with list change support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCapability {
    /// Support for list change notifications
    #[serde(default)]
    pub list_changed: bool,
}

/// Implementation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationInfo {
    /// Implementation name
    pub name: String,
    /// Implementation version
    pub version: String,
}

/// Represents the role of an MCP participant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// A host application that initiates connections
    Host,
    /// A connector within the host application
    Client,
    /// A service that provides context and capabilities
    Server,
}
