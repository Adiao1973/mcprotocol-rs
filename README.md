# mcprotocol-rs

[![Crates.io](https://img.shields.io/crates/v/mcprotocol-rs.svg)](https://crates.io/crates/mcprotocol-rs)
[![Documentation](https://docs.rs/mcprotocol-rs/badge.svg)](https://docs.rs/mcprotocol-rs)
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
  - 标准输入/输出传输
- 异步 API 设计
- 完整的类型安全
- 内置错误处理
- 可扩展的架构

- Complete implementation of MCP 2024-11-05 specification
- Multiple transport layer support
  - HTTP/SSE transport
  - Standard input/output transport
- Asynchronous API design
- Complete type safety
- Built-in error handling
- Extensible architecture

## 安装 | Installation

将以下内容添加到你的 `Cargo.toml`：
Add this to your `Cargo.toml`:

```toml
[dependencies]
mcprotocol-rs = "0.1.0"
```

## 快速开始 | Quick Start

### 作为客户端使用 | Using as a Client

```rust
use mcprotocol_rs::{
    client::ClientConfig,
    transport::{TransportConfig, TransportType},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置客户端 | Configure client
    let config = ClientConfig {
        name: "example-client".to_string(),
        version: "1.0.0".to_string(),
        roots: vec![],
    };

    // 配置传输层 | Configure transport
    let transport_config = TransportConfig {
        transport_type: TransportType::Http {
            base_url: "http://localhost:3000".to_string(),
            auth_token: None,
        },
        parameters: None,
    };

    // 初始化并运行 | Initialize and run
    // ... 实现具体的客户端逻辑 | Implement specific client logic

    Ok(())
}
```

### 作为服务器使用 | Using as a Server

```rust
use mcprotocol_rs::{
    server::{PromptManager, ResourceManager, ToolManager},
    transport::{TransportConfig, TransportType},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置服务器功能 | Configure server features
    // ... 实现 PromptManager, ResourceManager, ToolManager
    // ... Implement PromptManager, ResourceManager, ToolManager

    // 配置传输层 | Configure transport
    let transport_config = TransportConfig {
        transport_type: TransportType::Http {
            base_url: "http://0.0.0.0:3000".to_string(),
            auth_token: None,
        },
        parameters: None,
    };

    // 初始化并运行 | Initialize and run
    // ... 实现具体的服务器逻辑 | Implement specific server logic

    Ok(())
}
```

## 项目结构 | Project Structure

- `client/` - 客户端实现 | Client implementation
  - 根目录管理 | Root directory management
  - 采样请求处理 | Sampling request handling
- `server/` - 服务器实现 | Server implementation
  - 提示词管理 | Prompt management
  - 资源管理 | Resource management
  - 工具管理 | Tool management
- `protocol/` - MCP 协议实现 | MCP protocol implementation
  - 消息定义 | Message definitions
  - 生命周期管理 | Lifecycle management
  - 版本控制 | Version control
- `transport/` - 传输层实现 | Transport layer implementation
  - HTTP/SSE 传输 | HTTP/SSE transport
  - 标准输入/输出传输 | Standard input/output transport

## 贡献 | Contributing

欢迎提交 Pull Requests！对于重大更改，请先开 issue 讨论您想要更改的内容。

Pull Requests are welcome! For major changes, please open an issue first to discuss what you would like to change.

## 许可证 | License

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details