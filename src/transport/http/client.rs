use crate::{protocol::Message, Result};
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::{header, Client};
use serde_json;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// HTTP client configuration
/// HTTP 客户端配置
pub struct HttpClientConfig {
    pub base_url: String,
    pub auth_token: Option<String>,
}

/// HTTP client implementation
/// HTTP 客户端实现
pub struct HttpClient {
    config: HttpClientConfig,
    client: Client,
    message_endpoint: Arc<Mutex<Option<String>>>,
    receiver: Mutex<Option<mpsc::Receiver<Message>>>,
    client_id: Arc<Mutex<Option<String>>>,
}

impl HttpClient {
    /// Create a new HTTP client
    /// 创建一个新的 HTTP 客户端
    pub fn new(config: HttpClientConfig) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        if let Some(token) = &config.auth_token {
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|e| crate::Error::Transport(e.to_string()))?,
            );
        }

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| crate::Error::Transport(e.to_string()))?;

        Ok(Self {
            config,
            client,
            message_endpoint: Arc::new(Mutex::new(None)),
            receiver: Mutex::new(None),
            client_id: Arc::new(Mutex::new(None)),
        })
    }

    /// Wait for and get endpoint event
    /// 等待并获取 endpoint 事件
    fn wait_for_endpoint(event: &str) -> Option<(String, String)> {
        if event.trim().starts_with("event: endpoint\ndata:") {
            let data = event
                .lines()
                .find(|line| line.starts_with("data:"))
                .map(|line| line[5..].trim().to_string())?;

            // 解析 JSON 数据
            // Parse JSON data
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                let endpoint = json["endpoint"].as_str()?.to_string();
                let client_id = json["clientId"].as_str()?.to_string();
                return Some((endpoint, client_id));
            }
        }
        None
    }
}

#[async_trait]
impl super::HttpTransport for HttpClient {
    async fn initialize(&mut self) -> Result<()> {
        // Connect to SSE endpoint
        // 连接到 SSE 端点
        let url = format!("{}/events", self.config.base_url);
        let response = self
            .client
            .get(&url)
            .header(header::ACCEPT, "text/event-stream")
            .send()
            .await
            .map_err(|e| crate::Error::Transport(e.to_string()))?;

        // Create message receiving channel
        // 创建消息接收通道
        let (tx, rx) = mpsc::channel(32);
        *self.receiver.lock().unwrap() = Some(rx);

        // Handle SSE event stream
        // 处理 SSE 事件流
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let message_endpoint = Arc::clone(&self.message_endpoint);
        let client_id = Arc::clone(&self.client_id);

        tokio::spawn(async move {
            while let Some(Ok(chunk)) = stream.next().await {
                if let Ok(text) = String::from_utf8(chunk.to_vec()) {
                    buffer.push_str(&text);

                    // Process complete events
                    // 处理完整的事件
                    while let Some(event_end) = buffer.find("\n\n") {
                        let event = buffer[..event_end].to_string();
                        buffer.drain(..event_end + 2);

                        // Skip keepalive ping
                        // 跳过保活 ping
                        if event.trim() == "data: ping" {
                            continue;
                        }

                        // Handle endpoint event
                        // 处理 endpoint 事件
                        if event.contains("event: endpoint") {
                            if let Some((endpoint, id)) = HttpClient::wait_for_endpoint(&event) {
                                *message_endpoint.lock().unwrap() = Some(endpoint);
                                *client_id.lock().unwrap() = Some(id);
                                continue;
                            }
                        }

                        // Handle message event
                        // 处理消息事件
                        if event.contains("event: message") {
                            if let Some(data) =
                                event.lines().find(|line| line.starts_with("data: "))
                            {
                                let data = &data[6..];
                                if let Ok(message) = serde_json::from_str(data) {
                                    // Send all messages to the receiver channel
                                    // 发送所有消息到接收通道
                                    if tx.send(message).await.is_err() {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        // Wait for endpoint
        // 等待接收 endpoint
        let mut retries = 0;
        while self.message_endpoint.lock().unwrap().is_none() && retries < 10 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            retries += 1;
        }

        if self.message_endpoint.lock().unwrap().is_none() {
            return Err(crate::Error::Transport(
                "Failed to receive endpoint event".into(),
            ));
        }

        Ok(())
    }

    async fn send(&self, message: Message) -> Result<()> {
        let endpoint = self
            .message_endpoint
            .lock()
            .unwrap()
            .as_ref()
            .ok_or_else(|| crate::Error::Transport("Message endpoint not initialized".into()))?
            .clone();

        let client_id = self
            .client_id
            .lock()
            .unwrap()
            .as_ref()
            .ok_or_else(|| crate::Error::Transport("Client ID not initialized".into()))?
            .clone();

        self.client
            .post(&endpoint)
            .header("X-Client-ID", client_id)
            .json(&message)
            .send()
            .await
            .map_err(|e| crate::Error::Transport(e.to_string()))?
            .error_for_status()
            .map_err(|e| crate::Error::Transport(e.to_string()))?;

        Ok(())
    }

    async fn receive(&self) -> Result<Message> {
        let mut receiver = self
            .receiver
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| crate::Error::Transport("SSE connection not established".into()))?;

        let message = receiver
            .recv()
            .await
            .ok_or_else(|| crate::Error::Transport("SSE connection closed".into()))?;

        *self.receiver.lock().unwrap() = Some(receiver);
        Ok(message)
    }

    async fn close(&mut self) -> Result<()> {
        *self.message_endpoint.lock().unwrap() = None;
        *self.client_id.lock().unwrap() = None;
        *self.receiver.lock().unwrap() = None;
        Ok(())
    }
}

/// Default HTTP client type
/// 默认 HTTP 客户端类型
pub type DefaultHttpClient = HttpClient;
