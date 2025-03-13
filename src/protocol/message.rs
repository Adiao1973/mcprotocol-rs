use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

use super::{ClientCapabilities, ImplementationInfo, RequestId, ServerCapabilities};
use crate::Result;

/// Base JSON-RPC message
/// 基础 JSON-RPC 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Response(Response),
    Notification(Notification),
}

/// JSON-RPC request message
/// JSON-RPC 请求消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Protocol version (must be "2.0")
    /// 协议版本（必须为 "2.0"）
    pub jsonrpc: String,
    /// Request method
    /// 请求方法
    pub method: String,
    /// Optional parameters
    /// 可选参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Request ID
    /// 请求 ID
    pub id: RequestId,
}

/// JSON-RPC response message
/// JSON-RPC 响应消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Protocol version (must be "2.0")
    /// 协议版本（必须为 "2.0"）
    pub jsonrpc: String,
    /// Request ID
    /// 请求 ID
    pub id: RequestId,
    /// Response result
    /// 响应结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error if any
    /// 如果有错误
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

/// JSON-RPC notification message (request without ID)
/// JSON-RPC 通知消息（没有 ID 的请求）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Protocol version (must be "2.0")
    /// 协议版本（必须为 "2.0"）
    pub jsonrpc: String,
    /// Notification method
    /// 通知方法
    pub method: String,
    /// Optional parameters
    /// 可选参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Error response
/// 错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    /// Error code
    /// 错误代码
    pub code: i32,
    /// Error message
    /// 错误消息
    pub message: String,
    /// Additional error data
    /// 附加错误数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard error codes
/// 标准错误代码
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const SERVER_NOT_INITIALIZED: i32 = -32002;
    pub const UNKNOWN_ERROR_CODE: i32 = -32001;
    pub const REQUEST_CANCELLED: i32 = -32800;
}

/// MCP method types
/// MCP 方法类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Method {
    // Lifecycle methods
    // 生命周期方法
    Initialize,
    Initialized,
    Shutdown,
    Exit,

    // Utility methods
    // 实用方法
    #[serde(rename = "$/cancelRequest")]
    Cancel,
    #[serde(rename = "$/ping")]
    Ping,
    #[serde(rename = "$/pong")]
    Pong,
    #[serde(rename = "$/progress")]
    Progress,

    // Server feature methods
    // 服务器功能方法
    #[serde(rename = "prompts/list")]
    ListPrompts,
    #[serde(rename = "prompts/get")]
    GetPrompt,
    #[serde(rename = "prompts/execute")]
    ExecutePrompt,

    #[serde(rename = "resources/list")]
    ListResources,
    #[serde(rename = "resources/get")]
    GetResource,
    #[serde(rename = "resources/create")]
    CreateResource,
    #[serde(rename = "resources/update")]
    UpdateResource,
    #[serde(rename = "resources/delete")]
    DeleteResource,
    #[serde(rename = "resources/subscribe")]
    SubscribeResource,
    #[serde(rename = "resources/unsubscribe")]
    UnsubscribeResource,

    #[serde(rename = "tools/list")]
    ListTools,
    #[serde(rename = "tools/get")]
    GetTool,
    #[serde(rename = "tools/execute")]
    ExecuteTool,
    #[serde(rename = "tools/cancel")]
    CancelTool,

    // Client feature methods
    // 客户端功能方法
    #[serde(rename = "roots/list")]
    ListRoots,
    #[serde(rename = "roots/get")]
    GetRoot,

    #[serde(rename = "sampling/request")]
    SamplingRequest,
}

impl Request {
    /// Creates a new request
    /// 创建一个新的请求
    pub fn new(method: Method, params: Option<Value>, id: RequestId) -> Self {
        // ID is guaranteed to be string or number by type system
        // ID 已经通过类型系统保证是字符串或整数
        // ID uniqueness should be checked at session level
        // 在实际使用时，应该在会话级别检查 ID 的唯一性
        Self {
            jsonrpc: super::JSONRPC_VERSION.to_string(),
            method: method.to_string(),
            params,
            id,
        }
    }

    /// Validates that the request ID is unique within the given session
    /// 验证请求 ID 在给定的会话中是唯一的
    pub fn validate_id_uniqueness(&self, used_ids: &mut std::collections::HashSet<String>) -> bool {
        let id_str = match &self.id {
            RequestId::String(s) => s.clone(),
            RequestId::Number(n) => n.to_string(),
        };
        used_ids.insert(id_str)
    }
}

impl Response {
    /// Creates a new successful response
    /// 创建一个新的成功响应
    pub fn success(result: Value, id: RequestId) -> Self {
        Self {
            jsonrpc: super::JSONRPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Creates a new error response
    /// 创建一个新的错误响应
    pub fn error(error: ResponseError, id: RequestId) -> Self {
        Self {
            jsonrpc: super::JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

impl Notification {
    /// Creates a new notification
    /// 创建一个新的通知
    pub fn new(method: Method, params: Option<Value>) -> Self {
        Self {
            jsonrpc: super::JSONRPC_VERSION.to_string(),
            method: method.to_string(),
            params,
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::Initialize => write!(f, "initialize"),
            Method::Initialized => write!(f, "initialized"),
            Method::Shutdown => write!(f, "shutdown"),
            Method::Exit => write!(f, "exit"),
            Method::Cancel => write!(f, "$/cancelRequest"),
            Method::Ping => write!(f, "$/ping"),
            Method::Pong => write!(f, "$/pong"),
            Method::Progress => write!(f, "$/progress"),
            Method::ListPrompts => write!(f, "prompts/list"),
            Method::GetPrompt => write!(f, "prompts/get"),
            Method::ExecutePrompt => write!(f, "prompts/execute"),
            Method::ListResources => write!(f, "resources/list"),
            Method::GetResource => write!(f, "resources/get"),
            Method::CreateResource => write!(f, "resources/create"),
            Method::UpdateResource => write!(f, "resources/update"),
            Method::DeleteResource => write!(f, "resources/delete"),
            Method::SubscribeResource => write!(f, "resources/subscribe"),
            Method::UnsubscribeResource => write!(f, "resources/unsubscribe"),
            Method::ListTools => write!(f, "tools/list"),
            Method::GetTool => write!(f, "tools/get"),
            Method::ExecuteTool => write!(f, "tools/execute"),
            Method::CancelTool => write!(f, "tools/cancel"),
            Method::ListRoots => write!(f, "roots/list"),
            Method::GetRoot => write!(f, "roots/get"),
            Method::SamplingRequest => write!(f, "sampling/request"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_request_id_must_be_string_or_integer() {
        // test string id
        // 测试字符串 ID
        let string_id = RequestId::String("test-id".to_string());
        let request = Request::new(Method::Initialize, None, string_id.clone());
        assert!(matches!(request.id, RequestId::String(_)));

        // test integer id
        // 测试整数 ID
        let integer_id = RequestId::Number(42);
        let request = Request::new(Method::Initialize, None, integer_id.clone());
        assert!(matches!(request.id, RequestId::Number(_)));
    }

    #[test]
    fn test_request_id_uniqueness() {
        let mut used_ids = HashSet::new();

        // test string id uniqueness
        // 测试字符串 ID 的唯一性
        let id1 = RequestId::String("test-1".to_string());
        let id2 = RequestId::String("test-1".to_string());

        assert!(is_unique_id(&id1, &mut used_ids)); // First use should return true
                                                    // 第一次使用应该返回 true
        assert!(!is_unique_id(&id2, &mut used_ids)); // Repeated use should return false
                                                     // 重复使用应该返回 false

        // test integer id uniqueness
        // 测试整数 ID 的唯一性
        let id3 = RequestId::Number(1);
        let id4 = RequestId::Number(1);

        assert!(is_unique_id(&id3, &mut used_ids)); // First use should return true
                                                    // 第一次使用应该返回 true
        assert!(!is_unique_id(&id4, &mut used_ids)); // Repeated use should return false
                                                     // 重复使用应该返回 false
    }

    // Helper function: Check if ID is unique
    // 辅助函数：检查 ID 是否唯一
    fn is_unique_id(id: &RequestId, used_ids: &mut HashSet<String>) -> bool {
        let id_str = match id {
            RequestId::String(s) => s.clone(),
            RequestId::Number(n) => n.to_string(),
        };
        used_ids.insert(id_str)
    }

    #[test]
    fn test_request_id_serialization() {
        // test string id serialization
        // 测试字符串 ID 的序列化
        let string_id = RequestId::String("test-id".to_string());
        let json = serde_json::to_string(&string_id).unwrap();
        assert_eq!(json, r#""test-id""#);

        // test integer id serialization
        // 测试整数 ID 的序列化
        let integer_id = RequestId::Number(42);
        let json = serde_json::to_string(&integer_id).unwrap();
        assert_eq!(json, "42");
    }

    #[test]
    fn test_request_id_deserialization() {
        // test string id deserialization
        // 测试字符串 ID 的反序列化
        let json = r#""test-id""#;
        let id: RequestId = serde_json::from_str(json).unwrap();
        assert!(matches!(id, RequestId::String(s) if s == "test-id"));

        // test integer id deserialization
        // 测试整数 ID 的反序列化
        let json = "42";
        let id: RequestId = serde_json::from_str(json).unwrap();
        assert!(matches!(id, RequestId::Number(n) if n == 42));

        // test null value should fail
        // 测试 null 值应该失败
        let json = "null";
        let result: std::result::Result<RequestId, serde_json::Error> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_request_with_same_id_in_different_sessions() {
        // test same id in different sessions
        // 测试不同会话中使用相同的 ID
        let id = RequestId::Number(1);

        // first session
        // 第一个会话
        let mut session1_ids = HashSet::new();
        assert!(is_unique_id(&id, &mut session1_ids));

        // second session
        // 第二个会话（新的 HashSet 代表新的会话）
        let mut session2_ids = HashSet::new();
        assert!(is_unique_id(&id, &mut session2_ids));
    }
}
