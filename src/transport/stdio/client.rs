use crate::{protocol::Message, Result};
use async_trait::async_trait;
use std::{path::PathBuf, process::Stdio};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, Command},
    sync::Mutex,
};

/// Stdio client configuration
pub struct StdioClientConfig {
    /// Server executable path
    pub server_path: PathBuf,
    /// Server arguments
    pub server_args: Vec<String>,
    /// Buffer size
    pub buffer_size: usize,
    /// Whether to capture server logs
    pub capture_logs: bool,
}

impl Default for StdioClientConfig {
    fn default() -> Self {
        Self {
            server_path: PathBuf::from("mcp-server"),
            server_args: vec![],
            buffer_size: 4096,
            capture_logs: true,
        }
    }
}

/// Stdio client implementation
pub struct StdioClient {
    config: StdioClientConfig,
    child: Mutex<Option<Child>>,
    stdin: Mutex<Option<tokio::process::ChildStdin>>,
    stdout: Mutex<Option<BufReader<tokio::process::ChildStdout>>>,
    stderr: Mutex<Option<BufReader<tokio::process::ChildStderr>>>,
}

impl StdioClient {
    /// Create a new Stdio client
    pub fn new(config: StdioClientConfig) -> Self {
        Self {
            config,
            child: Mutex::new(None),
            stdin: Mutex::new(None),
            stdout: Mutex::new(None),
            stderr: Mutex::new(None),
        }
    }

    /// Start log capture
    async fn start_log_capture(&self, mut stderr: tokio::process::ChildStderr) {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }
                // Here you can handle logs as needed, such as forwarding to a specific logging system
                eprintln!("[MCP Server] {}", line.trim());
                line.clear();
            }
        });
    }
}

#[async_trait]
impl super::StdioTransport for StdioClient {
    async fn initialize(&mut self) -> Result<()> {
        let mut child = Command::new(&self.config.server_path)
            .args(&self.config.server_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(if self.config.capture_logs {
                Stdio::piped()
            } else {
                Stdio::inherit()
            })
            .spawn()
            .map_err(|e| crate::Error::Transport(format!("Failed to start server: {}", e)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| crate::Error::Transport("Failed to get server stdin handle".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| crate::Error::Transport("Failed to get server stdout handle".into()))?;

        if self.config.capture_logs {
            if let Some(stderr) = child.stderr.take() {
                self.start_log_capture(stderr).await;
            }
        }

        *self.stdin.lock().await = Some(stdin);
        *self.stdout.lock().await = Some(BufReader::new(stdout));
        *self.child.lock().await = Some(child);

        Ok(())
    }

    async fn send(&self, message: Message) -> Result<()> {
        let mut stdin = self.stdin.lock().await;
        let stdin = stdin
            .as_mut()
            .ok_or_else(|| crate::Error::Transport("Server process not initialized".into()))?;

        let json = serde_json::to_string(&message)?;
        if json.contains('\n') {
            return Err(crate::Error::Transport(
                "Message contains embedded newlines".into(),
            ));
        }

        stdin.write_all(json.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        Ok(())
    }

    async fn receive(&self) -> Result<Message> {
        let mut stdout = self.stdout.lock().await;
        let stdout = stdout
            .as_mut()
            .ok_or_else(|| crate::Error::Transport("Server process not initialized".into()))?;

        let mut line = String::with_capacity(self.config.buffer_size);
        stdout.read_line(&mut line).await?;

        if line.is_empty() {
            return Err(crate::Error::Transport("Server process terminated".into()));
        }

        let message = serde_json::from_str(&line)?;
        Ok(message)
    }

    async fn close(&mut self) -> Result<()> {
        let mut child = self.child.lock().await;
        if let Some(mut child) = child.take() {
            // First close stdin to let the server know there will be no more input
            drop(self.stdin.lock().await.take());

            // Wait for the server process to end
            match child.wait().await {
                Ok(status) => {
                    if !status.success() {
                        return Err(crate::Error::Transport(format!(
                            "Server process exited with status: {}",
                            status
                        )));
                    }
                }
                Err(e) => {
                    return Err(crate::Error::Transport(format!(
                        "Failed to wait for server process: {}",
                        e
                    )));
                }
            }
        }

        *self.stdout.lock().await = None;
        *self.stderr.lock().await = None;
        Ok(())
    }
}

/// Default Stdio client type
pub type DefaultStdioClient = StdioClient;
