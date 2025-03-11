use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::Mutex,
};

use async_trait::async_trait;
use serde_json::Value;

use crate::{protocol::Message, Result};

use super::Transport;

/// Standard IO transport implementation
pub struct StdioTransport {
    /// Child process handle
    child: Option<Child>,
    /// Input buffer for reading from stdout
    reader: Mutex<Option<BufReader<std::process::ChildStdout>>>,
    /// Output handle for writing to stdin
    writer: Mutex<Option<std::process::ChildStdin>>,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new(command: &str, args: &[&str]) -> Result<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdout = child.stdout.take().expect("Failed to get stdout");
        let stdin = child.stdin.take().expect("Failed to get stdin");

        Ok(Self {
            child: Some(child),
            reader: Mutex::new(Some(BufReader::new(stdout))),
            writer: Mutex::new(Some(stdin)),
        })
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn initialize(&mut self) -> Result<()> {
        // No initialization needed for stdio transport
        Ok(())
    }

    async fn send(&self, message: Message) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        let writer = writer.as_mut().expect("Transport not initialized");

        let json = serde_json::to_string(&message)?;
        writer.write_all(json.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;

        Ok(())
    }

    async fn receive(&self) -> Result<Message> {
        let mut reader = self.reader.lock().unwrap();
        let reader = reader.as_mut().expect("Transport not initialized");

        let mut line = String::new();
        reader.read_line(&mut line)?;

        let message: Message = serde_json::from_str(&line)?;
        Ok(message)
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            child.kill()?;
            child.wait()?;
        }
        Ok(())
    }
}
