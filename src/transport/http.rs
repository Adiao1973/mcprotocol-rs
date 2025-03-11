use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::stream::StreamExt;
use reqwest::{header, Client};
use tokio::sync::mpsc;

use crate::{protocol::Message, Result};

use super::Transport;

/// HTTP transport with SSE implementation
pub struct HttpTransport {
    /// Base URL for the HTTP server
    base_url: String,
    /// HTTP client
    client: Client,
    /// Message endpoint URL
    message_endpoint: Mutex<Option<String>>,
    /// SSE message receiver
    receiver: Arc<Mutex<Option<mpsc::Receiver<Message>>>>,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(base_url: String, auth_token: Option<String>) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        if let Some(token) = auth_token {
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|e| crate::Error::Transport(e.to_string()))?,
            );
        }

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            base_url,
            client,
            message_endpoint: Mutex::new(None),
            receiver: Arc::new(Mutex::new(None)),
        })
    }

    /// Start listening for SSE events
    async fn start_sse(&self) -> Result<()> {
        let url = format!("{}/events", self.base_url);
        let mut stream = self
            .client
            .get(&url)
            .header(header::ACCEPT, "text/event-stream")
            .send()
            .await
            .map_err(|e| crate::Error::Transport(e.to_string()))?
            .bytes_stream();

        let (tx, rx) = mpsc::channel(32);
        *self.receiver.lock().unwrap() = Some(rx);

        tokio::spawn(async move {
            while let Some(chunk) = stream.next().await {
                if let Ok(bytes) = chunk {
                    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                        for line in text.lines() {
                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                if let Ok(message) = serde_json::from_str(data) {
                                    let _ = tx.send(message).await;
                                }
                            } else if line.starts_with("event: endpoint") {
                                // Handle endpoint event
                                if let Some(endpoint) = line.split(": ").nth(1) {
                                    if let Ok(message) = serde_json::from_str(endpoint) {
                                        let _ = tx.send(message).await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn initialize(&mut self) -> Result<()> {
        self.start_sse().await?;
        Ok(())
    }

    async fn send(&self, message: Message) -> Result<()> {
        let endpoint = self
            .message_endpoint
            .lock()
            .unwrap()
            .as_ref()
            .ok_or_else(|| crate::Error::Protocol("Message endpoint not received".into()))?
            .clone();

        self.client
            .post(&endpoint)
            .json(&message)
            .send()
            .await
            .map_err(|e| crate::Error::Transport(e.to_string()))?;

        Ok(())
    }

    async fn receive(&self) -> Result<Message> {
        let mut receiver = self
            .receiver
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| crate::Error::Protocol("SSE connection not established".into()))?;

        let message = receiver
            .recv()
            .await
            .ok_or_else(|| crate::Error::Protocol("SSE connection closed".into()))?;

        *self.receiver.lock().unwrap() = Some(receiver);
        Ok(message)
    }

    async fn close(&mut self) -> Result<()> {
        // Clean up any resources
        *self.message_endpoint.lock().unwrap() = None;
        *self.receiver.lock().unwrap() = None;
        Ok(())
    }
}
