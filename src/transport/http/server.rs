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

/// HTTP server configuration
#[derive(Clone)]
pub struct HttpServerConfig {
    pub addr: SocketAddr,
    pub auth_token: Option<String>,
}

/// Axum HTTP server implementation
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
    /// Create a new Axum HTTP server
    pub fn new(config: HttpServerConfig) -> Self {
        let (tx, _) = broadcast::channel(32);
        Self { config, tx }
    }

    /// Create Axum router
    fn create_router(state: Arc<Self>) -> Router {
        Router::new()
            .route("/events", get(Self::sse_handler))
            .route("/messages", post(Self::message_handler))
            .with_state(state)
    }

    /// SSE event handler
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

    /// Message handler
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
        // Axum server will close automatically when dropped
        Ok(())
    }
}

/// Default HTTP server type
pub type DefaultHttpServer = AxumHttpServer;
