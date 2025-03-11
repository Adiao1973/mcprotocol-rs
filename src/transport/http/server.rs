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
use futures::stream::Stream;
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

/// HTTP 服务器配置
#[derive(Clone)]
pub struct HttpServerConfig {
    pub addr: SocketAddr,
    pub auth_token: Option<String>,
}

/// Axum HTTP 服务器实现
pub struct AxumHttpServer {
    config: HttpServerConfig,
    tx: broadcast::Sender<Message>,
}

impl Clone for AxumHttpServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            tx: self.tx.clone(),
        }
    }
}

impl AxumHttpServer {
    /// 创建新的 Axum HTTP 服务器
    pub fn new(config: HttpServerConfig) -> Self {
        let (tx, _) = broadcast::channel(32);
        Self { config, tx }
    }

    /// 创建 Axum 路由
    fn create_router(state: Arc<Self>) -> Router {
        Router::new()
            .route("/events", get(Self::sse_handler))
            .route("/messages", post(Self::message_handler))
            .with_state(state)
    }

    /// SSE 事件处理器
    async fn sse_handler(
        State(state): State<Arc<Self>>,
    ) -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
        let mut rx = state.tx.subscribe();
        let stream = async_stream::stream! {
            while let Ok(msg) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    yield Ok(Event::default().data(json));
                }
            }
        };

        Sse::new(stream)
    }

    /// 消息处理器
    async fn message_handler(
        State(state): State<Arc<Self>>,
        Json(message): Json<Message>,
    ) -> impl IntoResponse {
        match state.tx.send(message) {
            Ok(_) => (axum::http::StatusCode::OK, "Message sent").into_response(),
            Err(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to broadcast message: {}", e),
            )
                .into_response(),
        }
    }
}

#[async_trait]
impl super::HttpTransport for AxumHttpServer {
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

    async fn send(&self, message: Message) -> Result<()> {
        self.tx
            .send(message)
            .map_err(|e| crate::Error::Transport(e.to_string()))?;
        Ok(())
    }

    async fn receive(&self) -> Result<Message> {
        let mut rx = self.tx.subscribe();
        rx.recv()
            .await
            .map_err(|e| crate::Error::Transport(e.to_string()))
    }

    async fn close(&mut self) -> Result<()> {
        // Axum 服务器会在 drop 时自动关闭
        Ok(())
    }
}

/// 默认 HTTP 服务器类型
pub type DefaultHttpServer = AxumHttpServer;
