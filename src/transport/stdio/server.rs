use crate::{protocol::Message, Result};
use async_trait::async_trait;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::Mutex,
};

/// Stdio server configuration
pub struct StdioServerConfig {
    /// Buffer size
    pub buffer_size: usize,
}

impl Default for StdioServerConfig {
    fn default() -> Self {
        Self { buffer_size: 4096 }
    }
}

/// Stdio server implementation
pub struct StdioServer {
    config: StdioServerConfig,
    stdin: Mutex<BufReader<tokio::io::Stdin>>,
    stdout: Mutex<tokio::io::Stdout>,
}

impl StdioServer {
    /// Create a new Stdio server
    pub fn new(config: StdioServerConfig) -> Self {
        let stdin = BufReader::new(tokio::io::stdin());
        let stdout = tokio::io::stdout();

        Self {
            config,
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(stdout),
        }
    }

    /// Log a message (using stderr)
    pub async fn log(&self, message: &str) -> Result<()> {
        let mut stderr = tokio::io::stderr();
        stderr.write_all(message.as_bytes()).await?;
        stderr.write_all(b"\n").await?;
        stderr.flush().await?;
        Ok(())
    }
}

#[async_trait]
impl super::StdioTransport for StdioServer {
    async fn initialize(&mut self) -> Result<()> {
        self.log("MCP server initialized").await?;
        Ok(())
    }

    async fn send(&self, message: Message) -> Result<()> {
        let mut stdout = self.stdout.lock().await;
        let json = serde_json::to_string(&message)?;

        // Check if the message contains a newline
        if json.contains('\n') {
            self.log("Warning: Message contains embedded newlines")
                .await?;
            return Err(crate::Error::Transport(
                "Message contains embedded newlines".into(),
            ));
        }

        stdout.write_all(json.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
        Ok(())
    }

    async fn receive(&self) -> Result<Message> {
        let mut stdin = self.stdin.lock().await;
        let mut line = String::with_capacity(self.config.buffer_size);

        if stdin.read_line(&mut line).await? == 0 {
            self.log("Client connection closed").await?;
            return Err(crate::Error::Transport("Client connection closed".into()));
        }

        match serde_json::from_str(&line) {
            Ok(message) => Ok(message),
            Err(e) => {
                self.log(&format!("Error parsing message: {}", e)).await?;
                Err(crate::Error::Transport(format!(
                    "Invalid message format: {}",
                    e
                )))
            }
        }
    }

    async fn close(&mut self) -> Result<()> {
        self.log("MCP server shutting down").await?;
        Ok(())
    }
}

/// Default Stdio server type
pub type DefaultStdioServer = StdioServer;
