use async_trait::async_trait;
use serde_json::Value;

use crate::{protocol::Message, Result};

pub mod http;
pub mod stdio;

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Transport type
    pub transport_type: TransportType,
    /// Optional configuration parameters
    pub parameters: Option<Value>,
}

/// Transport type
#[derive(Debug, Clone)]
pub enum TransportType {
    /// Standard IO transport
    Stdio,
    /// HTTP with SSE transport
    Http {
        /// Base URL for the HTTP server
        base_url: String,
        /// Optional authentication token
        auth_token: Option<String>,
    },
}

/// Transport trait for MCP communication
#[async_trait]
pub trait Transport: Send + Sync {
    /// Initialize the transport
    async fn initialize(&mut self) -> Result<()>;

    /// Send a message
    async fn send(&self, message: Message) -> Result<()>;

    /// Receive a message
    async fn receive(&self) -> Result<Message>;

    /// Close the transport
    async fn close(&mut self) -> Result<()>;
}

/// Transport factory for creating transport instances
pub trait TransportFactory {
    /// Create a new transport instance
    fn create(&self, config: TransportConfig) -> Result<Box<dyn Transport>>;
}
