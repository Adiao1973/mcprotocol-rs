use mcprotocol_rs::{
    transport::{ClientTransportFactory, TransportConfig, TransportType},
    ClientCapabilities, ImplementationInfo, Message, Method, Notification, Request, RequestId,
    Result, PROTOCOL_VERSION,
};
use serde_json::json;
use std::{collections::HashSet, env};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // 跟踪会话中使用的请求 ID
    // Track request IDs used in the session
    let mut session_ids = HashSet::new();

    // 获取服务器程序路径
    // Get server program path
    let server_path = env::current_dir()?.join("target/debug/examples/lifecycle_server");

    // 配置 Stdio 客户端
    // Configure Stdio client
    let config = TransportConfig {
        transport_type: TransportType::Stdio {
            server_path: Some(server_path.to_str().unwrap().to_string()),
            server_args: None,
        },
        parameters: None,
    };

    // 创建客户端实例
    // Create client instance
    let factory = ClientTransportFactory;
    let mut client = factory.create(config)?;

    eprintln!("Client starting...");

    // 初始化客户端
    // Initialize client
    client.initialize().await?;

    // 发送初始化请求
    // Send initialize request
    let init_request = Request::new(
        Method::Initialize,
        Some(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": ClientCapabilities {
                roots: None,
                sampling: None,
                experimental: None,
            },
            "clientInfo": ImplementationInfo {
                name: "Example Client".to_string(),
                version: "1.0.0".to_string(),
            }
        })),
        RequestId::Number(1),
    );

    // 验证请求 ID 的唯一性
    // Validate request ID uniqueness
    if !init_request.validate_id_uniqueness(&mut session_ids) {
        eprintln!("Request ID has already been used in this session");
        return Ok(());
    }

    eprintln!("Sending initialize request...");
    client.send(Message::Request(init_request)).await?;

    // 等待初始化响应
    // Wait for initialize response
    match client.receive().await {
        Ok(message) => {
            match message {
                Message::Response(response) => {
                    if response.error.is_some() {
                        eprintln!("Initialization failed: {:?}", response.error);
                        return Ok(());
                    }

                    if let Some(result) = response.result {
                        // 检查服务器版本和能力
                        // Check server version and capabilities
                        let server_version = result
                            .get("protocolVersion")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");

                        if server_version != PROTOCOL_VERSION {
                            eprintln!(
                                "Protocol version mismatch: expected {}, got {}",
                                PROTOCOL_VERSION, server_version
                            );
                            return Ok(());
                        }

                        eprintln!("Server initialized with version: {}", server_version);

                        // 发送初始化完成通知
                        // Send initialized notification
                        let init_notification = Notification::new(Method::Initialized, None);
                        client
                            .send(Message::Notification(init_notification))
                            .await?;
                        eprintln!("Sent initialized notification");

                        // 模拟一些操作
                        // Simulate some operations
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                        // 发送关闭请求
                        // Send shutdown request
                        eprintln!("Sending shutdown request...");
                        let shutdown_request =
                            Request::new(Method::Shutdown, None, RequestId::Number(2));

                        // 验证请求 ID 的唯一性
                        // Validate request ID uniqueness
                        if !shutdown_request.validate_id_uniqueness(&mut session_ids) {
                            eprintln!("Request ID has already been used in this session");
                            return Ok(());
                        }

                        client.send(Message::Request(shutdown_request)).await?;

                        // 等待关闭响应
                        // Wait for shutdown response
                        match client.receive().await {
                            Ok(message) => {
                                match message {
                                    Message::Response(response) => {
                                        if response.error.is_some() {
                                            eprintln!("Shutdown failed: {:?}", response.error);
                                            return Ok(());
                                        }

                                        // 发送退出通知
                                        // Send exit notification
                                        eprintln!("Sending exit notification...");
                                        let exit_notification =
                                            Notification::new(Method::Exit, None);
                                        client
                                            .send(Message::Notification(exit_notification))
                                            .await?;
                                    }
                                    _ => eprintln!("Unexpected response type"),
                                }
                            }
                            Err(e) => {
                                eprintln!("Error receiving response: {}", e);
                                return Ok(());
                            }
                        }
                    }
                }
                _ => eprintln!("Unexpected message type"),
            }
        }
        Err(e) => {
            eprintln!("Error receiving response: {}", e);
            return Ok(());
        }
    }

    // 关闭客户端
    // Close client
    client.close().await?;
    eprintln!("Client stopped");
    Ok(())
}
