use crate::{protocol::Message, Result};
use async_trait::async_trait;

pub mod client;
pub mod server;

/// HTTP transport trait
#[async_trait]
pub trait HttpTransport: Send + Sync {
    /// Initialize the transport
    async fn initialize(&mut self) -> Result<()>;
    /// Send a message
    async fn send(&self, message: Message) -> Result<()>;
    /// Receive a message
    async fn receive(&self) -> Result<Message>;
    /// Close the connection
    async fn close(&mut self) -> Result<()>;
}

// Re-export default implementations
pub use self::client::DefaultHttpClient;
pub use self::server::DefaultHttpServer;
