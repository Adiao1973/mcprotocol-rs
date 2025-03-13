# mcprotocol-rs

[![Crates.io](https://img.shields.io/crates/v/mcprotocol-rs.svg)](https://crates.io/crates/mcprotocol-rs)
[![GitHub](https://img.shields.io/github/stars/Adiao1973/mcprotocol-rs?style=social)](https://github.com/Adiao1973/mcprotocol-rs)
[![Documentation](https://docs.rs/mcprotocol-rs/badge.svg)](https://docs.rs/crate/mcprotocol-rs/latest)
[![License:MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> ⚠️ **开发状态**: 本项目目前处于积极开发中，API 可能会发生变化。
> 
> ⚠️ **Development Status**: This project is under active development and the API may change.

`mcprotocol-rs` 是 Model Context Protocol (MCP) 的 Rust 实现。它提供了一个完整的框架来实现 MCP 客户端和服务器。

`mcprotocol-rs` is a Rust implementation of the Model Context Protocol (MCP). It provides a complete framework for implementing MCP clients and servers.

## 特性 | Features

- 完整实现 MCP 2024-11-05 规范
- 支持多种传输层
  - HTTP/SSE 传输
    - 基于 axum 的高性能服务器实现
    - 支持 SSE 实时消息推送
    - 内置认证支持
  - 标准输入/输出传输
    - 符合 MCP 规范的子进程管理
    - 支持服务器日志捕获
    - 自动处理进程生命周期
- 异步 API 设计
  - 基于 tokio 的异步运行时
  - 完整的 Stream 支持
  - 非阻塞 I/O 操作
- 完整的类型安全
- 内置错误处理
- 可扩展的架构
  - 模块化的传输层设计
  - 支持自定义传输实现
  - 工厂模式创建实例

- Complete implementation of MCP 2024-11-05 specification
- Multiple transport layer support
  - HTTP/SSE transport
    - High-performance server implementation based on axum
    - SSE real-time message push support
    - Built-in authentication support
  - Standard input/output transport
    - MCP-compliant subprocess management
    - Server log capture support
    - Automatic process lifecycle handling
- Asynchronous API design
  - Based on tokio runtime
  - Complete Stream support
  - Non-blocking I/O operations
- Complete type safety
- Built-in error handling
- Extensible architecture
  - Modular transport layer design
  - Custom transport implementation support
  - Factory pattern for instance creation

## 安装 | Installation

将以下内容添加到你的 `Cargo.toml`：
Add this to your `Cargo.toml`:

```toml
[dependencies]
mcprotocol-rs = "0.1.2"
```

## 快速开始 | Quick Start

### HTTP 服务器示例 | HTTP Server Example

```rust
use mcprotocol_rs::{
    transport::{
        ServerTransportFactory,
        TransportConfig,
        TransportType,
    },
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 配置 HTTP 服务器
    let config = TransportConfig {
        transport_type: TransportType::Http {
            base_url: "127.0.0.1:3000".to_string(),
            auth_token: Some("your-auth-token".to_string()),
        },
        parameters: None,
    };

    // 使用工厂创建服务器
    let factory = ServerTransportFactory;
    let mut server = factory.create(config)?;
    
    // 初始化并启动服务器
    server.initialize().await?;

    // 保持服务器运行
    tokio::signal::ctrl_c().await?;
    Ok(())
}
```

### Stdio 传输示例 | Stdio Transport Example

首先创建服务器程序 `examples/stdio_server.rs`：
First create the server program `examples/stdio_server.rs`:

```rust
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
                if let Message::Request(request) = message {
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
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }
    }

    server.close().await?;
    Ok(())
}
```

然后创建客户端程序 `examples/stdio_client.rs`：
Then create the client program `examples/stdio_client.rs`:

```rust
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
            if let Message::Response(resp) = response {
                if let Some(result) = resp.result {
                    eprintln!("Server response: {}", result);
                }
            }
        }
        Err(e) => {
            eprintln!("Error receiving response: {}", e);
        }
    }

    client.close().await?;
    Ok(())
}
```

运行示例 | Running the example:

```bash
# 1. 首先编译服务器程序 | First, build the server
cargo build --example stdio_server

# 2. 然后运行客户端程序 | Then run the client
cargo run --example stdio_client
```

客户端会自动启动服务器进程并通过标准输入/输出进行通信。
The client will automatically start the server process and communicate through stdin/stdout.

## 自定义传输实现 | Custom Transport Implementation

你可以通过实现 `Transport` trait 来创建自己的传输层：

You can create your own transport layer by implementing the `Transport` trait:

```rust
use mcprotocol_rs::{
    transport::Transport,
    protocol::Message,
    Result,
};
use async_trait::async_trait;

#[derive(Clone)]
struct MyTransport {
    // 你的传输层字段
    // Your transport fields
}

#[async_trait]
impl Transport for MyTransport {
    async fn initialize(&mut self) -> Result<()> {
        // 实现初始化逻辑
        // Implement initialization logic
    }

    async fn send(&self, message: Message) -> Result<()> {
        // 实现发送逻辑
        // Implement send logic
    }

    async fn receive(&self) -> Result<Message> {
        // 实现接收逻辑
        // Implement receive logic
    }

    async fn close(&mut self) -> Result<()> {
        // 实现关闭逻辑
        // Implement close logic
    }
}
```

## 项目结构 | Project Structure

```
src/
├── client/           # 客户端实现 | Client implementation
├── server/           # 服务器实现 | Server implementation
├── protocol/         # MCP 协议实现 | MCP protocol implementation
└── transport/        # 传输层实现 | Transport layer implementation
    ├── http/         # HTTP/SSE 传输 | HTTP/SSE transport
    │   ├── client.rs # HTTP 客户端 | HTTP client
    │   └── server.rs # HTTP 服务器 | HTTP server
    └── stdio/        # 标准输入/输出传输 | Stdio transport
        ├── client.rs # Stdio 客户端 | Stdio client
        └── server.rs # Stdio 服务器 | Stdio server
```

## 贡献 | Contributing

欢迎提交 Pull Requests！对于重大更改，请先开 issue 讨论您想要更改的内容。

Pull Requests are welcome! For major changes, please open an issue first to discuss what you would like to change.

## 许可证 | License

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details