use mcprotocol_rs::message;
use mcprotocol_rs::{
    protocol::{Message, Response},
    transport::{ServerTransportFactory, TransportConfig, TransportType},
    Result,
};
use serde_json::json;
use std::collections::HashSet;

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

    // 初始化服务器
    // Initialize server
    server.initialize().await?;
    eprintln!("Server initialized and ready to receive messages...");

    // 持续接收和处理消息
    // Continuously receive and process messages
    loop {
        match server.receive().await {
            Ok(message) => {
                eprintln!("Received message: {:?}", message);

                // 根据消息类型处理
                // Process messages based on type
                match message {
                    Message::Request(request) => {
                        // 验证请求 ID 的唯一性
                        // Validate request ID uniqueness
                        if !request.validate_id_uniqueness(&mut session_ids) {
                            let error = Message::Response(Response::error(
                                message::ResponseError {
                                    code: message::error_codes::INVALID_REQUEST,
                                    message: "Request ID has already been used".to_string(),
                                    data: None,
                                },
                                request.id,
                            ));
                            if let Err(e) = server.send(error).await {
                                eprintln!("Error sending error response: {}", e);
                                break;
                            }
                            continue;
                        }

                        match request.method.as_str() {
                            "prompts/execute" => {
                                // 创建响应消息
                                // Create response message
                                let response = Message::Response(Response::success(
                                    json!({
                                        "content": "Hello from server!",
                                        "role": "assistant"
                                    }),
                                    request.id,
                                ));

                                // 发送响应
                                // Send response
                                if let Err(e) = server.send(response).await {
                                    eprintln!("Error sending response: {}", e);
                                    break;
                                }
                            }
                            _ => {
                                eprintln!("Unknown method: {}", request.method);
                                let error = Message::Response(Response::error(
                                    message::ResponseError {
                                        code: message::error_codes::METHOD_NOT_FOUND,
                                        message: "Method not found".to_string(),
                                        data: None,
                                    },
                                    request.id,
                                ));
                                if let Err(e) = server.send(error).await {
                                    eprintln!("Error sending error response: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    _ => {
                        eprintln!("Unexpected message type");
                    }
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
    Ok(())
}
