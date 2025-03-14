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
    - 自动管理客户端连接生命周期
    - 精确的消息路由机制
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
    - Automatic client connection lifecycle management
    - Precise message routing mechanism
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
mcprotocol-rs = "0.1.5"
```

## 快速开始 | Quick Start

### HTTP/SSE 服务器示例 | HTTP/SSE Server Example

```rust
use mcprotocol_rs::{
    transport::{
        ServerTransportFactory,
        TransportConfig,
        TransportType,
    },
    Result,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    // 配置 HTTP 服务器
    // Configure HTTP server
    let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();
    let config = TransportConfig {
        transport_type: TransportType::Http {
            base_url: addr.to_string(),
            auth_token: Some("your-auth-token".to_string()),
        },
        parameters: None,
    };

    // 使用工厂创建服务器
    // Create server using factory
    let factory = ServerTransportFactory;
    let mut server = factory.create(config)?;
    
    // 初始化并启动服务器
    // Initialize and start server
    server.initialize().await?;
    println!("Server started on {}", addr);

    // 保持服务器运行
    // Keep server running
    tokio::signal::ctrl_c().await?;
    Ok(())
}
```

### HTTP/SSE 客户端示例 | HTTP/SSE Client Example

```rust
use mcprotocol_rs::{
    transport::{ClientTransportFactory, TransportConfig, TransportType},
    protocol::{Message, Method, Request, RequestId},
    Result,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // 配置 HTTP 客户端
    // Configure HTTP client
    let config = TransportConfig {
        transport_type: TransportType::Http {
            base_url: "http://127.0.0.1:3000".to_string(),
            auth_token: Some("your-auth-token".to_string()),
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
    println!("Client connected to server");

    // 创建并发送请求
    // Create and send request
    let request = Request::new(
        Method::Ping,
        None,
        RequestId::String("ping-1".to_string()),
    );
    client.send(Message::Request(request)).await?;

    // 接收响应
    // Receive response
    let response = client.receive().await?;
    println!("Received response: {:?}", response);

    // 关闭客户端
    // Close client
    client.close().await?;
    Ok(())
}
```

### Stdio 传输示例 | Stdio Transport Example

[查看完整示例代码](examples/stdio_example.rs)
[See full example code](examples/stdio_example.rs)

### 生命周期示例 | Lifecycle Example

[查看完整示例代码](examples/lifecycle_example.rs)
[See full example code](examples/lifecycle_example.rs)

## HTTP/SSE 传输特性 | HTTP/SSE Transport Features

### 客户端管理 | Client Management

- 基于 SSE 的客户端连接管理
- 自动清理断开的连接
- 保持连接活跃检测
- 支持客户端重连机制

- SSE-based client connection management
- Automatic cleanup of disconnected clients
- Keep-alive connection detection
- Support for client reconnection

### 消息路由 | Message Routing

- 基于请求 ID 的消息路由
- 支持请求-响应模式
- 支持通知消息
- 自动清理断开的连接

- Request ID based message routing
- Support for request-response pattern
- Support for notification messages
- Automatic cleanup of disconnected connections

### 安全性 | Security

- 支持 Bearer Token 认证
- 安全的消息传输
- 连接状态监控

- Bearer Token authentication support
- Secure message transmission
- Connection state monitoring

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
├── protocol/         # MCP 协议实现 | MCP protocol implementation
├── transport/        # 传输层实现 | Transport layer implementation
│   ├── http/        # HTTP/SSE 传输 | HTTP/SSE transport
│   │   ├── client.rs # HTTP 客户端 | HTTP client
│   │   └── server.rs # HTTP 服务器 | HTTP server
│   └── stdio/       # 标准输入/输出传输 | Stdio transport
│       ├── client.rs # Stdio 客户端 | Stdio client
│       └── server.rs # Stdio 服务器 | Stdio server
├── client_features/ # 客户端特性实现 | Client features implementation
├── server_features/ # 服务器特性实现 | Server features implementation
├── error.rs        # 错误类型定义 | Error type definitions
└── lib.rs          # 库入口和导出 | Library entry and exports

examples/
├── ping_example.rs      # Ping/Pong 示例 | Ping/Pong example
├── lifecycle_client.rs  # 生命周期客户端示例 | Lifecycle client example
├── lifecycle_server.rs  # 生命周期服务器示例 | Lifecycle server example
├── stdio_client.rs      # 标准输入/输出客户端示例 | Stdio client example
└── stdio_server.rs      # 标准输入/输出服务器示例 | Stdio server example
```

## 贡献 | Contributing

欢迎提交 Pull Requests！对于重大更改，请先开 issue 讨论您想要更改的内容。

Pull Requests are welcome! For major changes, please open an issue first to discuss what you would like to change.

## 许可证 | License

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details