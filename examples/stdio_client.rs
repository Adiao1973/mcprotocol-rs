use mcprotocol_rs::{
    protocol::{Message, Method, Request, RequestId},
    transport::{ClientTransportFactory, TransportConfig, TransportType},
    Result,
};
use serde_json::json;
use std::{collections::HashSet, env, time::Duration};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // 跟踪会话中使用的请求 ID
    // Track request IDs used in the session
    let mut session_ids = HashSet::new();

    // 获取服务器程序路径
    // Get server program path
    let server_path = env::current_dir()?.join("target/debug/examples/stdio_server");

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

    // 初始化客户端
    // Initialize client
    client.initialize().await?;
    eprintln!("Client initialized and connected to server...");

    // 等待服务器初始化完成
    // Wait for server initialization to complete
    sleep(Duration::from_millis(100)).await;

    // 创建请求
    // Create request
    let request_id = RequestId::Number(1);
    let request = Request::new(
        Method::ExecutePrompt,
        Some(json!({
            "content": "Hello from client!",
            "role": "user"
        })),
        request_id,
    );

    // 验证请求 ID 的唯一性
    // Validate request ID uniqueness
    if !request.validate_id_uniqueness(&mut session_ids) {
        eprintln!("Request ID has already been used in this session");
        return Ok(());
    }

    // 发送消息
    // Send message
    eprintln!("Sending message to server...");
    client.send(Message::Request(request)).await?;

    // 接收服务器响应
    // Receive server response
    match client.receive().await {
        Ok(response) => {
            eprintln!("Received response: {:?}", response);
            match response {
                Message::Response(resp) => {
                    if let Some(result) = resp.result {
                        eprintln!("Server response result: {}", result);
                    }
                    if let Some(error) = resp.error {
                        eprintln!(
                            "Server response error: {} (code: {})",
                            error.message, error.code
                        );
                    }
                }
                _ => eprintln!("Unexpected response type"),
            }
        }
        Err(e) => {
            eprintln!("Error receiving response: {}", e);
        }
    }

    // 关闭客户端
    // Close client
    client.close().await?;
    Ok(())
}
