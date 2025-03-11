pub mod client_features;
pub mod error;
pub mod protocol;
pub mod server_features;
pub mod transport;

pub use client_features::*;
pub use error::Error;
pub use protocol::*;
pub use server_features::*;
pub use transport::*;

/// Result type for MCP operations
pub type Result<T> = std::result::Result<T, Error>;
