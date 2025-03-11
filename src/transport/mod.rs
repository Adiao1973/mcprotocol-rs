use async_trait::async_trait;
use serde_json::Value;

use crate::transport::{http::HttpTransport, stdio::StdioTransport};
use crate::{protocol::Message, Result};

pub mod http;
pub mod stdio;

// Re-export default implementations
pub use http::{client::DefaultHttpClient as HttpClient, server::DefaultHttpServer as HttpServer};
pub use stdio::{
    client::DefaultStdioClient as StdioClient, server::DefaultStdioServer as StdioServer,
};

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
    /// Stdio transport
    Stdio {
        /// Server executable path (only required for clients)
        server_path: Option<String>,
        /// Server arguments (only required for clients)
        server_args: Option<Vec<String>>,
    },
    /// HTTP transport
    Http {
        /// Server base URL
        base_url: String,
        /// Optional authentication token
        auth_token: Option<String>,
    },
}

/// Base trait for transport layers
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

/// Client transport factory
pub struct ClientTransportFactory;

impl ClientTransportFactory {
    /// Create a new transport instance
    pub fn create(&self, config: TransportConfig) -> Result<Box<dyn Transport>> {
        match config.transport_type {
            TransportType::Stdio {
                server_path,
                server_args,
            } => {
                use stdio::client::{StdioClient, StdioClientConfig};
                let config = StdioClientConfig {
                    server_path: server_path
                        .map(std::path::PathBuf::from)
                        .unwrap_or_default(),
                    server_args: server_args.unwrap_or_default(),
                    ..Default::default()
                };
                let client = StdioClient::new(config);
                Ok(Box::new(StdioClientTransport(client)))
            }
            TransportType::Http {
                base_url,
                auth_token,
            } => {
                use http::client::{HttpClient, HttpClientConfig};
                let config = HttpClientConfig {
                    base_url,
                    auth_token,
                };
                let client = HttpClient::new(config)?;
                Ok(Box::new(HttpClientTransport(client)))
            }
        }
    }
}

/// Server transport factory
pub struct ServerTransportFactory;

impl ServerTransportFactory {
    /// Create a new transport instance
    pub fn create(&self, config: TransportConfig) -> Result<Box<dyn Transport>> {
        match config.transport_type {
            TransportType::Stdio { .. } => {
                use stdio::server::{StdioServer, StdioServerConfig};
                let server = StdioServer::new(StdioServerConfig::default());
                Ok(Box::new(StdioServerTransport(server)))
            }
            TransportType::Http {
                base_url,
                auth_token,
            } => {
                use http::server::{AxumHttpServer, HttpServerConfig};
                let addr = base_url
                    .parse()
                    .map_err(|e| crate::Error::Transport(format!("Invalid address: {}", e)))?;
                let config = HttpServerConfig { addr, auth_token };
                let server = AxumHttpServer::new(config);
                Ok(Box::new(HttpServerTransport(server)))
            }
        }
    }
}

// 包装类型，用于实现 Transport trait
struct StdioClientTransport(stdio::client::StdioClient);
struct StdioServerTransport(stdio::server::StdioServer);
struct HttpClientTransport(http::client::HttpClient);
struct HttpServerTransport(http::server::AxumHttpServer);

// 为包装类型实现 Transport trait
macro_rules! impl_transport {
    ($wrapper:ident, $inner:ident) => {
        #[async_trait]
        impl Transport for $wrapper {
            async fn initialize(&mut self) -> Result<()> {
                self.0.initialize().await
            }

            async fn send(&self, message: Message) -> Result<()> {
                self.0.send(message).await
            }

            async fn receive(&self) -> Result<Message> {
                self.0.receive().await
            }

            async fn close(&mut self) -> Result<()> {
                self.0.close().await
            }
        }
    };
}

impl_transport!(StdioClientTransport, StdioClient);
impl_transport!(StdioServerTransport, StdioServer);
impl_transport!(HttpClientTransport, HttpClient);
impl_transport!(HttpServerTransport, AxumHttpServer);
