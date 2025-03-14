use crate::protocol::{RequestId, Response};
use crate::{protocol::Message, Result};
use async_trait::async_trait;
use axum::{
    extract::State,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use futures::{
    channel::mpsc,
    stream::{Stream, StreamExt},
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

/// Client ID type
/// 客户端 ID 类型
type ClientId = u64;

/// Client information
/// 客户端信息
#[derive(Clone)]
struct ClientInfo {
    /// Message sender channel
    /// 消息发送通道
    sender: MessageSender,
    /// Last request ID from this client
    /// 该客户端的最后一个请求 ID
    last_request_id: Option<RequestId>,
}

/// Message sender channel type
/// 消息发送通道类型
type MessageSender = mpsc::UnboundedSender<Message>;

/// HTTP server configuration
/// HTTP 服务器配置
#[derive(Clone)]
pub struct HttpServerConfig {
    /// Server address
    /// 服务器地址
    pub addr: SocketAddr,
    /// Optional authentication token
    /// 可选的认证令牌
    pub auth_token: Option<String>,
}

/// Axum HTTP server implementation
/// Axum HTTP 服务器实现
pub struct AxumHttpServer {
    /// Server configuration
    /// 服务器配置
    config: HttpServerConfig,
    /// Connected clients map
    /// 已连接客户端映射
    clients: Arc<Mutex<HashMap<ClientId, ClientInfo>>>,
    /// Next client ID counter
    /// 下一个客户端 ID 计数器
    next_client_id: Arc<AtomicU64>,
}

impl Clone for AxumHttpServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            clients: self.clients.clone(),
            next_client_id: self.next_client_id.clone(),
        }
    }
}

impl AxumHttpServer {
    /// Create a new Axum HTTP server
    /// 创建新的 Axum HTTP 服务器
    pub fn new(config: HttpServerConfig) -> Self {
        Self {
            config,
            clients: Arc::new(Mutex::new(HashMap::new())),
            next_client_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Create Axum router
    /// 创建 Axum 路由器
    fn create_router(state: Arc<Self>) -> Router {
        Router::new()
            .route("/events", get(Self::sse_handler))
            .route("/messages", post(Self::message_handler))
            .with_state(state)
    }

    /// SSE event handler
    /// SSE 事件处理器
    async fn sse_handler(
        State(state): State<Arc<Self>>,
    ) -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
        // Create a channel for the new client
        // 为新客户端创建通道
        let (tx, rx) = mpsc::unbounded();
        let client_id = state.next_client_id.fetch_add(1, Ordering::SeqCst);

        // Store client information
        // 存储客户端信息
        let client_info = ClientInfo {
            sender: tx,
            last_request_id: None,
        };
        state.clients.lock().await.insert(client_id, client_info);

        // Create cleanup function
        // 创建清理函数
        let clients = state.clients.clone();
        let stream = async_stream::stream! {
            // Send initial endpoint event
            // 发送初始端点事件
            let endpoint = format!("http://{}/messages", state.config.addr);
            yield Ok(Event::default()
                .event("endpoint")
                .data(endpoint));

            // Forward all messages until connection closes
            // 转发所有消息直到连接关闭
            let mut rx = rx;
            while let Some(msg) = rx.next().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    yield Ok(Event::default()
                        .event("message")
                        .data(json));
                }
            }

            // Remove client when stream ends (client disconnects)
            // 当流结束时移除客户端（客户端断开连接）
            clients.lock().await.remove(&client_id);
        };

        Sse::new(stream).keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(1))
                .text("ping"),
        )
    }

    /// Find the client that sent the request
    /// 查找发送请求的客户端
    async fn find_client_by_request_id(&self, request_id: &RequestId) -> Option<ClientId> {
        let clients = self.clients.lock().await;
        for (client_id, info) in clients.iter() {
            if let Some(last_request_id) = &info.last_request_id {
                if last_request_id == request_id {
                    return Some(*client_id);
                }
            }
        }
        None
    }

    /// Message handler
    /// 消息处理器
    async fn message_handler(
        State(state): State<Arc<Self>>,
        Json(message): Json<Message>,
    ) -> impl IntoResponse {
        match &message {
            Message::Request(request) => {
                // Find the most recently active client
                // 查找最近活动的客户端
                let client_id = {
                    let mut clients = state.clients.lock().await;
                    clients.iter_mut().next().map(|(id, _)| *id)
                };

                // If client found, update its last request ID
                // 如果找到客户端，更新其最后请求 ID
                if let Some(client_id) = client_id {
                    if let Some(client_info) = state.clients.lock().await.get_mut(&client_id) {
                        client_info.last_request_id = Some(request.id.clone());
                    }
                }

                let response = match request.method.as_str() {
                    "ping" => {
                        // Create pong response
                        // 创建 pong 响应
                        Response::success(json!({}), request.id.clone())
                    }
                    "shutdown" => {
                        // Create shutdown response
                        // 创建关闭响应
                        Response::success(json!(null), request.id.clone())
                    }
                    _ => {
                        // Create method not found error response
                        // 创建方法未找到错误响应
                        Response::error(
                            crate::protocol::ResponseError {
                                code: crate::error_codes::METHOD_NOT_FOUND,
                                message: "Method not found".to_string(),
                                data: None,
                            },
                            request.id.clone(),
                        )
                    }
                };

                // Send response to the most recently active client
                // 向最近活动的客户端发送响应
                if let Some(client_id) = client_id {
                    if let Some(client_info) = state.clients.lock().await.get(&client_id) {
                        let _ = client_info
                            .sender
                            .unbounded_send(Message::Response(response));
                    }
                }
            }
            Message::Notification(notification) => {
                if notification.method.as_str() == "exit" {
                    // Clean up all client connections
                    // 清理所有客户端连接
                    state.clients.lock().await.clear();
                }
                // Notifications don't need responses
                // 通知消息不需要响应
            }
            _ => {
                // Ignore other types of messages
                // 忽略其他类型的消息
            }
        }

        // Return success response
        // 返回成功响应
        (axum::http::StatusCode::OK, "Message sent").into_response()
    }

    /// Send message to a specific client
    /// 发送消息给指定的客户端
    async fn send_to_client(&self, client_id: ClientId, message: Message) -> Result<()> {
        if let Some(client_info) = self.clients.lock().await.get(&client_id) {
            client_info
                .sender
                .unbounded_send(message)
                .map_err(|e| crate::Error::Transport(e.to_string()))?;
        }
        Ok(())
    }
}

#[async_trait]
impl super::HttpTransport for AxumHttpServer {
    /// Initialize the server
    /// 初始化服务器
    async fn initialize(&mut self) -> Result<()> {
        let app = Self::create_router(Arc::new(self.clone()));
        let addr = self.config.addr;

        tokio::spawn(async move {
            axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
                .await
                .unwrap();
        });

        Ok(())
    }

    /// Send a message
    /// 发送消息
    async fn send(&self, message: Message) -> Result<()> {
        match &message {
            Message::Response(response) => {
                // Send response only to the client that sent the request
                // 只向发送请求的客户端发送响应
                if let Some(client_id) = self.find_client_by_request_id(&response.id).await {
                    self.send_to_client(client_id, message).await?;
                }
            }
            Message::Notification(_) => {
                // Send notifications to all clients
                // 通知消息发送给所有客户端
                let clients = self.clients.lock().await;
                for (client_id, _) in clients.iter() {
                    self.send_to_client(*client_id, message.clone()).await?;
                }
            }
            _ => {
                // Ignore other types of messages
                // 忽略其他类型的消息
            }
        }
        Ok(())
    }

    /// Receive a message
    /// 接收消息
    async fn receive(&self) -> Result<Message> {
        // Server doesn't need to implement receive as it receives messages via HTTP POST
        // 服务器不需要实现 receive，因为它通过 HTTP POST 接收消息
        Err(crate::Error::Transport(
            "Server does not support direct message receiving. Use HTTP POST endpoint instead."
                .into(),
        ))
    }

    /// Close the server
    /// 关闭服务器
    async fn close(&mut self) -> Result<()> {
        // Clean up all client connections
        // 清理所有客户端连接
        self.clients.lock().await.clear();
        Ok(())
    }
}

/// Default HTTP server type
/// 默认 HTTP 服务器类型
pub type DefaultHttpServer = AxumHttpServer;
