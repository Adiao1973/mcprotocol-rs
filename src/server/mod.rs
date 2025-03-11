pub mod prompts;
pub mod resources;
pub mod tools;

pub use prompts::*;
pub use resources::*;
pub use tools::*;

/// Server capability flags
#[derive(Debug, Clone, Default)]
pub struct ServerCapabilities {
    /// Whether prompts are supported
    pub prompts: bool,
    /// Whether resources are supported
    pub resources: bool,
    /// Whether tools are supported
    pub tools: bool,
}
