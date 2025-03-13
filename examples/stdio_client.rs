use mcprotocol_rs::{
    protocol::{Message, Method, Request, RequestId},
    transport::{ClientTransportFactory, TransportConfig, TransportType},
    Result,
};
use serde_json::json;
use std::{env, time::Duration};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // 获取服务器程序路径
    let server_path = env::current_dir()?.join("target/debug/examples/stdio_server");

    // 配置 Stdio 客户端
    let config = TransportConfig {
        transport_type: TransportType::Stdio {
            server_path: Some(server_path.to_str().unwrap().to_string()),
            server_args: None,
        },
        parameters: None,
    };

    // 创建客户端实例
    let factory = ClientTransportFactory;
    let mut client = factory.create(config)?;

    // 初始化客户端
    client.initialize().await?;
    eprintln!("Client initialized and connected to server...");

    // 等待服务器初始化完成
    sleep(Duration::from_millis(100)).await;

    // 创建并发送消息
    let request_id = RequestId::Number(1);
    let message = Message::Request(Request::new(
        Method::ExecutePrompt,
        Some(json!({
            "content": "Hello from client!",
            "role": "user"
        })),
        request_id,
    ));

    eprintln!("Sending message to server...");
    client.send(message).await?;

    // 接收服务器响应
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
    client.close().await?;
    Ok(())
}
