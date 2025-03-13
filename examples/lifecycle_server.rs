use mcprotocol_rs::{
    error_codes,
    protocol::ServerCapabilities,
    transport::{ServerTransportFactory, TransportConfig, TransportType},
    ImplementationInfo, Message, Response, ResponseError, Result, PROTOCOL_VERSION,
};
use serde_json::json;
use std::collections::HashSet;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // 跟踪会话中使用的请求 ID
    // Track request IDs used in the session
    let mut session_ids = HashSet::new();

    // 配置 Stdio 服务器
    // Configure Stdio server
    let config = TransportConfig {
        transport_type: TransportType::Stdio {
            server_path: None,
            server_args: None,
        },
        parameters: None,
    };

    // 创建服务器实例
    // Create server instance
    let factory = ServerTransportFactory;
    let mut server = factory.create(config)?;
    let mut initialized = false;

    // 启动服务器
    // Start server
    eprintln!("Server starting...");
    server.initialize().await?;

    // 处理消息循环
    // Message handling loop
    loop {
        match server.receive().await {
            Ok(message) => {
                match message {
                    Message::Request(request) => {
                        // 验证请求 ID 的唯一性
                        // Validate request ID uniqueness
                        if !request.validate_id_uniqueness(&mut session_ids) {
                            let error = ResponseError {
                                code: error_codes::INVALID_REQUEST,
                                message: "Request ID has already been used".to_string(),
                                data: None,
                            };
                            let response = Response::error(error, request.id);
                            server.send(Message::Response(response)).await?;
                            continue;
                        }

                        match request.method.as_str() {
                            "initialize" => {
                                // 处理初始化请求
                                // Handle initialize request
                                eprintln!("Received initialize request");

                                // 解析客户端能力和版本
                                // Parse client capabilities and version
                                if let Some(params) = request.params {
                                    let client_version = params
                                        .get("protocolVersion")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown");

                                    // 版本检查
                                    // Version check
                                    if client_version != PROTOCOL_VERSION {
                                        // 发送错误响应
                                        // Send error response
                                        let error = ResponseError {
                                            code: error_codes::INVALID_REQUEST,
                                            message: "Unsupported protocol version".to_string(),
                                            data: Some(json!({
                                                "supported": [PROTOCOL_VERSION],
                                                "requested": client_version
                                            })),
                                        };
                                        let response = Response::error(error, request.id);
                                        server.send(Message::Response(response)).await?;
                                        continue;
                                    }

                                    // 发送成功响应
                                    // Send success response
                                    let response = Response::success(
                                        json!({
                                            "protocolVersion": PROTOCOL_VERSION,
                                            "capabilities": ServerCapabilities {
                                                prompts: None,
                                                resources: None,
                                                tools: None,
                                                logging: Some(json!({})),
                                                experimental: None,
                                            },
                                            "serverInfo": ImplementationInfo {
                                                name: "Example Server".to_string(),
                                                version: "1.0.0".to_string(),
                                            }
                                        }),
                                        request.id,
                                    );
                                    server.send(Message::Response(response)).await?;
                                }
                            }
                            "shutdown" => {
                                if !initialized {
                                    // 如果未初始化，发送错误
                                    // If not initialized, send error
                                    let error = ResponseError {
                                        code: error_codes::SERVER_NOT_INITIALIZED,
                                        message: "Server not initialized".to_string(),
                                        data: None,
                                    };
                                    let response = Response::error(error, request.id);
                                    server.send(Message::Response(response)).await?;
                                    continue;
                                }

                                // 发送成功响应
                                // Send success response
                                let response = Response::success(json!(null), request.id);
                                server.send(Message::Response(response)).await?;

                                // 等待退出通知
                                // Wait for exit notification
                                eprintln!("Server shutting down...");
                                break;
                            }
                            _ => {
                                if !initialized {
                                    // 如果未初始化，拒绝其他请求
                                    // If not initialized, reject other requests
                                    let error = ResponseError {
                                        code: error_codes::SERVER_NOT_INITIALIZED,
                                        message: "Server not initialized".to_string(),
                                        data: None,
                                    };
                                    let response = Response::error(error, request.id);
                                    server.send(Message::Response(response)).await?;
                                }
                            }
                        }
                    }
                    Message::Notification(notification) => match notification.method.as_str() {
                        "initialized" => {
                            eprintln!("Server initialized");
                            initialized = true;
                        }
                        "exit" => {
                            eprintln!("Received exit notification");
                            break;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }
    }

    // 关闭服务器
    // Close server
    server.close().await?;
    eprintln!("Server stopped");
    Ok(())
}
