use mcprotocol_rs::{
    error_codes,
    transport::{ClientTransportFactory, ServerTransportFactory, TransportConfig, TransportType},
    Message, Method, Notification, Request, RequestId, Response, ResponseError, Result,
};
use serde_json::json;
use std::{collections::HashSet, time::Duration};
use tokio::{self, time::sleep, time::timeout};

const PING_INTERVAL: Duration = Duration::from_secs(5);
const PING_TIMEOUT: Duration = Duration::from_secs(2);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const SERVER_PORT: u16 = 3000;
const SERVER_URL: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() -> Result<()> {
    // 启动服务器
    // Start server
    let server_handle = tokio::spawn(run_server());

    // 等待服务器启动
    // Wait for server to start
    sleep(Duration::from_millis(100)).await;

    // 启动客户端
    // Start client
    let client_handle = tokio::spawn(run_client());

    // 等待客户端和服务器完成
    // Wait for client and server to complete
    match tokio::try_join!(server_handle, client_handle) {
        Ok((server_result, client_result)) => {
            server_result?;
            client_result?;
            Ok(())
        }
        Err(e) => {
            eprintln!("Error in task execution: {}", e);
            Err(mcprotocol_rs::Error::Transport(e.to_string()))
        }
    }
}

async fn run_server() -> Result<()> {
    // 配置服务器
    // Configure server
    let config = TransportConfig {
        transport_type: TransportType::Http {
            base_url: SERVER_URL.to_string(),
            auth_token: None,
        },
        parameters: None,
    };

    // 创建服务器实例
    // Create server instance
    let factory = ServerTransportFactory;
    let mut server = factory.create(config)?;

    // 初始化服务器
    // Initialize server
    server.initialize().await?;
    eprintln!(
        "Server started and waiting for ping requests on port {}",
        SERVER_PORT
    );

    // 等待退出信号
    // Wait for exit signal
    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

    let exit_signal = async move {
        rx.await.ok();
    };

    tokio::select! {
        _ = exit_signal => {
            eprintln!("Server received exit signal");
        }
        _ = tokio::time::sleep(Duration::from_secs(30)) => {
            eprintln!("Server timeout after 30 seconds");
        }
    }

    server.close().await?;
    eprintln!("Server stopped");
    Ok(())
}

async fn run_client() -> Result<()> {
    // 跟踪会话中使用的请求 ID
    // Track request IDs used in the session
    let mut session_ids = HashSet::new();
    let mut ping_count = 0;

    // 配置客户端
    // Configure client
    let config = TransportConfig {
        transport_type: TransportType::Http {
            base_url: format!("http://{}", SERVER_URL),
            auth_token: None,
        },
        parameters: None,
    };

    // 创建客户端实例
    // Create client instance
    let factory = ClientTransportFactory;
    let mut client = factory.create(config)?;

    // 初始化客户端
    // Initialize client
    match timeout(CONNECTION_TIMEOUT, client.initialize()).await {
        Ok(result) => result?,
        Err(_) => {
            return Err(mcprotocol_rs::Error::Transport(
                "Client initialization timeout".into(),
            ))
        }
    }
    eprintln!("Client started");

    // 发送 3 次 ping 请求
    // Send 3 ping requests
    while ping_count < 3 {
        // 发送 ping 请求
        // Send ping request
        let request_id = RequestId::String(format!("ping-{}", ping_count + 1));
        let ping_request = Request::new(Method::Ping, None, request_id.clone());

        // 验证请求 ID 的唯一性
        // Validate request ID uniqueness
        if !ping_request.validate_id_uniqueness(&mut session_ids) {
            eprintln!("Request ID has already been used in this session");
            break;
        }

        eprintln!("Sending ping request #{}", ping_count + 1);
        client.send(Message::Request(ping_request.clone())).await?;

        // 等待 pong 响应，带超时
        // Wait for pong response with timeout
        let response = timeout(PING_TIMEOUT, client.receive()).await;
        match response {
            Ok(Ok(Message::Response(response))) => {
                // 验证响应 ID 是否匹配
                // Verify response ID matches
                if !request_id_matches(&request_id, &response.id) {
                    eprintln!(
                        "Received response with mismatched ID: expected {}, got {}",
                        request_id_to_string(&request_id),
                        request_id_to_string(&response.id)
                    );
                    continue;
                }

                if response.error.is_some() {
                    eprintln!("Received error response: {:?}", response.error);
                    break;
                }
                eprintln!("Received pong response #{}", ping_count + 1);
            }
            Ok(Ok(message)) => {
                eprintln!("Unexpected message type: {:?}", message);
                continue;
            }
            Ok(Err(e)) => {
                eprintln!("Error receiving response: {}", e);
                break;
            }
            Err(_) => {
                eprintln!("Ping timeout for request #{}", ping_count + 1);
                break;
            }
        }

        ping_count += 1;
        if ping_count < 3 {
            sleep(PING_INTERVAL).await;
        }
    }

    // 发送关闭请求
    // Send shutdown request
    let shutdown_request = Request::new(
        Method::Shutdown,
        None,
        RequestId::String("shutdown".to_string()),
    );

    // 验证请求 ID 的唯一性
    // Validate request ID uniqueness
    if !shutdown_request.validate_id_uniqueness(&mut session_ids) {
        eprintln!("Request ID has already been used in this session");
        return Ok(());
    }

    client.send(Message::Request(shutdown_request)).await?;

    // 等待关闭响应
    // Wait for shutdown response
    match timeout(PING_TIMEOUT, client.receive()).await {
        Ok(Ok(Message::Response(response))) => {
            if response.error.is_some() {
                eprintln!("Shutdown failed: {:?}", response.error);
                return Ok(());
            }
            // 发送退出通知
            // Send exit notification
            let exit_notification = Notification::new(Method::Exit, None);
            client
                .send(Message::Notification(exit_notification))
                .await?;
        }
        Ok(Ok(_)) => {
            eprintln!("Unexpected response type");
            return Ok(());
        }
        Ok(Err(e)) => {
            eprintln!("Error receiving shutdown response: {}", e);
            return Ok(());
        }
        Err(_) => {
            eprintln!("Shutdown response timeout");
            return Ok(());
        }
    }

    client.close().await?;
    eprintln!("Client stopped");
    Ok(())
}

// 辅助函数：检查请求 ID 是否匹配
// Helper function: Check if request ID matches
fn request_id_matches(request_id: &RequestId, response_id: &RequestId) -> bool {
    match (request_id, response_id) {
        (RequestId::String(req), RequestId::String(res)) => req == res,
        (RequestId::Number(req), RequestId::Number(res)) => req == res,
        _ => false,
    }
}

// 辅助函数：将请求 ID 转换为字符串
// Helper function: Convert request ID to string
fn request_id_to_string(id: &RequestId) -> String {
    match id {
        RequestId::String(s) => s.clone(),
        RequestId::Number(n) => n.to_string(),
    }
}
