use crate::{protocol::Message, Result};
use async_trait::async_trait;
use reqwest::{header, Client};
use std::sync::Mutex;
use tokio::sync::mpsc;

/// HTTP client configuration
pub struct HttpClientConfig {
    pub base_url: String,
    pub auth_token: Option<String>,
}

/// HTTP client implementation
pub struct HttpClient {
    config: HttpClientConfig,
    client: Client,
    message_endpoint: Mutex<Option<String>>,
    receiver: Mutex<Option<mpsc::Receiver<Message>>>,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new(config: HttpClientConfig) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        if let Some(token) = &config.auth_token {
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|e| crate::Error::Transport(e.to_string()))?,
            );
        }

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            config,
            client,
            message_endpoint: Mutex::new(None),
            receiver: Mutex::new(None),
        })
    }
}

#[async_trait]
impl super::HttpTransport for HttpClient {
    async fn initialize(&mut self) -> Result<()> {
        let url = format!("{}/events", self.config.base_url);
        let stream = self
            .client
            .get(&url)
            .header(header::ACCEPT, "text/event-stream")
            .send()
            .await
            .map_err(|e| crate::Error::Transport(e.to_string()))?;

        // Handle SSE connection...
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
        *self.message_endpoint.lock().unwrap() = None;
        *self.receiver.lock().unwrap() = None;
        Ok(())
    }
}

/// Default HTTP client type
pub type DefaultHttpClient = HttpClient;
