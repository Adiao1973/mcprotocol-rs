use mcprotocol_rs::message;
use mcprotocol_rs::{
    protocol::{Message, Response},
    transport::{ServerTransportFactory, TransportConfig, TransportType},
    Result,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // 配置 Stdio 服务器
    let config = TransportConfig {
        transport_type: TransportType::Stdio {
            server_path: None,
            server_args: None,
        },
        parameters: None,
    };

    // 创建服务器实例
    let factory = ServerTransportFactory;
    let mut server = factory.create(config)?;

    // 初始化服务器
    server.initialize().await?;
    eprintln!("Server initialized and ready to receive messages...");

    // 持续接收和处理消息
    loop {
        match server.receive().await {
            Ok(message) => {
                eprintln!("Received message: {:?}", message);

                // 根据消息类型处理
                match message {
                    Message::Request(request) => {
                        match request.method.as_str() {
                            "prompts/execute" => {
                                // 创建响应消息
                                let response = Message::Response(Response::success(
                                    json!({
                                        "content": "Hello from server!",
                                        "role": "assistant"
                                    }),
                                    request.id,
                                ));

                                // 发送响应
                                if let Err(e) = server.send(response).await {
                                    eprintln!("Error sending response: {}", e);
                                    break;
                                }
                            }
                            _ => {
                                eprintln!("Unknown method: {}", request.method);
                                let error = Message::Response(Response::error(
                                    message::ResponseError {
                                        code: -32601,
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
    server.close().await?;
    Ok(())
}
