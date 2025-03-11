pub mod client;
pub mod error;
pub mod protocol;
pub mod server;
pub mod transport;

pub use client::*;
pub use error::Error;
pub use protocol::*;
pub use server::*;
pub use transport::*;

/// Result type for MCP operations
pub type Result<T> = std::result::Result<T, Error>;
