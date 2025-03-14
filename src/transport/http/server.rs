use crate::protocol::{RequestId, Response};
use crate::{protocol::Message, Error, Result};
use async_trait::async_trait;
use axum::{
    extract::State,
    http::StatusCode,
    middleware::{self, Next},
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
    /// Client connection time
    /// 客户端连接时间
    connected_at: std::time::Instant,
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

    /// Validate Bearer token from request headers
    /// 验证请求头中的 Bearer token
    fn validate_auth_token(
        headers: &axum::http::HeaderMap,
        auth_token: &Option<String>,
    ) -> Result<()> {
        if let Some(expected_token) = auth_token {
            match headers.get("Authorization") {
                Some(auth_header) => {
                    let auth_str = auth_header
                        .to_str()
                        .map_err(|_| Error::Transport("Invalid authorization header".into()))?;

                    if !auth_str.starts_with("Bearer ") {
                        return Err(Error::Transport("Invalid authorization format".into()));
                    }

                    let token = &auth_str["Bearer ".len()..];
                    if token != expected_token {
                        return Err(Error::Transport("Invalid token".into()));
                    }
                }
                None => return Err(Error::Transport("Missing authorization header".into())),
            }
        }
        Ok(())
    }

    /// Authentication middleware
    /// 认证中间件
    async fn auth_middleware(
        State(auth_token): State<Option<String>>,
        headers: axum::http::HeaderMap,
        request: axum::http::Request<axum::body::Body>,
        next: Next,
    ) -> impl IntoResponse {
        match Self::validate_auth_token(&headers, &auth_token) {
            Ok(_) => Ok(next.run(request).await),
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    }

    /// Create Axum router
    /// 创建 Axum 路由器
    fn create_router(state: Arc<Self>) -> Router {
        let auth_token = state.config.auth_token.clone();

        Router::new()
            .route("/events", get(Self::sse_handler))
            .route("/messages", post(Self::message_handler))
            .layer(middleware::from_fn_with_state(
                auth_token.clone(),
                Self::auth_middleware,
            ))
            .with_state(state)
    }

    /// Check and remove inactive clients
    /// 检查并移除不活跃的客户端
    async fn cleanup_inactive_clients(&self) {
        let now = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(300); // 5 minutes timeout

        let mut clients = self.clients.lock().await;
        clients.retain(|_, info| {
            let is_active = now.duration_since(info.connected_at) < timeout;
            is_active
        });
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
            connected_at: std::time::Instant::now(),
        };
        state.clients.lock().await.insert(client_id, client_info);

        // Start periodic cleanup
        // 启动定期清理
        let state_clone = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                state_clone.cleanup_inactive_clients().await;
            }
        });

        // Create cleanup function
        // 创建清理函数
        let clients = state.clients.clone();
        let stream = async_stream::stream! {
            // Send initial endpoint event with client ID
            // 发送带有客户端 ID 的初始端点事件
            let endpoint = format!("http://{}/messages", state.config.addr);
            yield Ok(Event::default()
                .event("endpoint")
                .data(format!("{{\"endpoint\":\"{}\",\"clientId\":\"{}\"}}", endpoint, client_id)));

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
        headers: axum::http::HeaderMap,
        Json(message): Json<Message>,
    ) -> impl IntoResponse {
        // Get client ID from request headers
        // 从请求头中获取客户端 ID
        let client_id = headers
            .get("X-Client-ID")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        // Update client's last activity time
        // 更新客户端的最后活动时间
        if let Some(client_id) = client_id {
            if let Some(client_info) = state.clients.lock().await.get_mut(&client_id) {
                client_info.connected_at = std::time::Instant::now();
            }
        }

        match &message {
            Message::Request(request) => {
                if let Some(client_id) = client_id {
                    // 更新客户端的最后请求 ID
                    // Update client's last request ID
                    if let Some(client_info) = state.clients.lock().await.get_mut(&client_id) {
                        client_info.last_request_id = Some(request.id.clone());
                    }

                    let response = match request.method.as_str() {
                        "ping" => {
                            // 创建 pong 响应
                            // Create pong response
                            Response::success(json!({}), request.id.clone())
                        }
                        "shutdown" => {
                            // 创建关闭响应
                            // Create shutdown response
                            Response::success(json!(null), request.id.clone())
                        }
                        _ => {
                            // 创建方法未找到错误响应
                            // Create method not found error response
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

                    // 向发送请求的客户端发送响应
                    // Send response to the requesting client
                    if let Some(client_info) = state.clients.lock().await.get(&client_id) {
                        let _ = client_info
                            .sender
                            .unbounded_send(Message::Response(response));
                    }
                }
            }
            Message::Notification(notification) => {
                if notification.method.as_str() == "exit" {
                    // 清理所有客户端连接
                    // Clean up all client connections
                    state.clients.lock().await.clear();
                }
                // 通知消息不需要响应
                // Notifications don't need responses
            }
            _ => {
                // 忽略其他类型的消息
                // Ignore other types of messages
            }
        }

        // 返回成功响应
        // Return success response
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
