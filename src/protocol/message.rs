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
    #[serde(rename = "notifications/cancelled")]
    Cancel,
    #[serde(rename = "ping")]
    Ping,
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
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Creates a new error response
    /// 创建一个新的错误响应
    pub fn error(error: ResponseError, id: RequestId) -> Self {
        Self {
            jsonrpc: super::JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(error),
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
            Method::Cancel => write!(f, "notifications/cancelled"),
            Method::Ping => write!(f, "ping"),
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
    use serde_json::json;
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

    #[test]
    fn test_response_must_match_request_id() {
        // Create a request
        // 创建一个请求
        let request_id = RequestId::Number(42);
        let request = Request::new(Method::Initialize, None, request_id.clone());

        // Create success response
        // 创建成功响应
        let success_response = Response::success(json!({"result": "success"}), request_id.clone());
        assert!(matches!(success_response.id, RequestId::Number(42)));

        // Create error response
        // 创建错误响应
        let error_response = Response::error(
            ResponseError {
                code: error_codes::INTERNAL_ERROR,
                message: "error".to_string(),
                data: None,
            },
            request_id.clone(),
        );
        assert!(matches!(error_response.id, RequestId::Number(42)));

        // Verify response with different ID
        // 验证不同 ID 的响应
        let different_id = RequestId::Number(43);
        let different_response = Response::success(json!({"result": "success"}), different_id);
        assert!(matches!(different_response.id, RequestId::Number(43)));
    }

    #[test]
    fn test_response_must_set_result_or_error_not_both() {
        let id = RequestId::Number(1);

        // Test success response only sets result
        // 测试成功响应只设置 result
        let success_response = Response::success(json!({"data": "success"}), id.clone());
        assert!(success_response.result.is_some());
        assert!(success_response.error.is_none());

        // Test error response only sets error
        // 测试错误响应只设置 error
        let error_response = Response::error(
            ResponseError {
                code: error_codes::INTERNAL_ERROR,
                message: "error".to_string(),
                data: None,
            },
            id.clone(),
        );
        assert!(error_response.result.is_none());
        assert!(error_response.error.is_some());

        // Test serialization ensures both are not included
        // 测试序列化确保不会同时包含两者
        let success_json = serde_json::to_string(&success_response).unwrap();
        assert!(!success_json.contains(r#""error""#));

        let error_json = serde_json::to_string(&error_response).unwrap();
        assert!(!error_json.contains(r#""result""#));
    }

    #[test]
    fn test_error_code_must_be_integer() {
        let id = RequestId::Number(1);

        // Test standard error codes
        // 测试标准错误代码
        let standard_errors = [
            error_codes::PARSE_ERROR,
            error_codes::INVALID_REQUEST,
            error_codes::METHOD_NOT_FOUND,
            error_codes::INVALID_PARAMS,
            error_codes::INTERNAL_ERROR,
            error_codes::SERVER_NOT_INITIALIZED,
            error_codes::UNKNOWN_ERROR_CODE,
            error_codes::REQUEST_CANCELLED,
        ];

        for &code in &standard_errors {
            let error_response = Response::error(
                ResponseError {
                    code,
                    message: "test error".to_string(),
                    data: None,
                },
                id.clone(),
            );

            if let Some(error) = error_response.error {
                // Verify error code is integer type
                // 验证错误代码是整数类型
                assert_eq!(
                    std::mem::size_of_val(&error.code),
                    std::mem::size_of::<i32>()
                );

                // Verify error code serialization
                // 验证错误代码的序列化
                let json = serde_json::to_string(&error).unwrap();
                assert!(json.contains(&format!(r#"code":{}"#, error.code)));
            } else {
                panic!("Error field should be set");
            }
        }

        // Test custom error codes
        // 测试自定义错误代码
        let custom_codes = [-1, 0, 1, 1000, -1000];
        for code in custom_codes {
            let error_response = Response::error(
                ResponseError {
                    code,
                    message: "custom error".to_string(),
                    data: None,
                },
                id.clone(),
            );

            if let Some(error) = error_response.error {
                assert_eq!(error.code, code);
                // Verify error code is integer type
                // 验证错误代码是整数类型
                assert_eq!(
                    std::mem::size_of_val(&error.code),
                    std::mem::size_of::<i32>()
                );
            } else {
                panic!("Error field should be set");
            }
        }
    }

    #[test]
    fn test_notification_must_not_contain_id() {
        // Create a notification
        // 创建一个通知
        let notification = Notification::new(Method::Initialized, Some(json!({"status": "ready"})));

        // Serialize notification to JSON
        // 将通知序列化为 JSON
        let json_str = serde_json::to_string(&notification).unwrap();

        // Verify JSON does not contain "id" field
        // 验证 JSON 不包含 "id" 字段
        assert!(!json_str.contains(r#""id""#));

        // Test deserialization of notification without ID
        // 测试不带 ID 的通知的反序列化
        let json_without_id = r#"{
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {"status": "ready"}
        }"#;
        let parsed: Message = serde_json::from_str(json_without_id).unwrap();
        assert!(matches!(parsed, Message::Notification(_)));

        // Test message with ID should not be parsed as notification
        // 测试带有 ID 的消息不应该被解析为通知
        let json_with_id = r#"{
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {"status": "ready"},
            "id": 1
        }"#;
        let parsed: Message = serde_json::from_str(json_with_id).unwrap();
        assert!(matches!(parsed, Message::Request(_)));
        assert!(!matches!(parsed, Message::Notification(_)));
    }

    #[test]
    fn test_initialization_protocol_compliance() {
        // Test initialize request format
        // 测试初始化请求格式
        let request = Request::new(
            Method::Initialize,
            Some(json!({
                "protocolVersion": super::super::PROTOCOL_VERSION,
                "capabilities": {
                    "roots": {
                        "listChanged": true
                    },
                    "sampling": {}
                },
                "clientInfo": {
                    "name": "TestClient",
                    "version": "1.0.0"
                }
            })),
            RequestId::Number(1),
        );

        let request_json = serde_json::to_string(&request).unwrap();

        // Verify request format
        // 验证请求格式
        assert!(request_json.contains(r#""method":"initialize""#));
        assert!(request_json.contains(super::super::PROTOCOL_VERSION));
        assert!(request_json.contains(r#""capabilities""#));
        assert!(request_json.contains(r#""clientInfo""#));

        // Test initialize response format
        // 测试初始化响应格式
        let response = Response::success(
            json!({
                "protocolVersion": super::super::PROTOCOL_VERSION,
                "capabilities": {
                    "prompts": {
                        "listChanged": true
                    },
                    "resources": {
                        "subscribe": true,
                        "listChanged": true
                    },
                    "tools": {
                        "listChanged": true
                    },
                    "logging": {}
                },
                "serverInfo": {
                    "name": "TestServer",
                    "version": "1.0.0"
                }
            }),
            RequestId::Number(1),
        );

        let response_json = serde_json::to_string(&response).unwrap();

        // Verify response format
        // 验证响应格式
        assert!(response_json.contains(super::super::PROTOCOL_VERSION));
        assert!(response_json.contains(r#""capabilities""#));
        assert!(response_json.contains(r#""serverInfo""#));

        // Test initialized notification format
        // 测试初始化完成通知格式
        let notification = Notification::new(Method::Initialized, None);
        let notification_json = serde_json::to_string(&notification).unwrap();

        // Verify notification format
        // 验证通知格式
        assert!(notification_json.contains(r#""method":"initialized""#));
        assert!(!notification_json.contains(r#""id""#));
    }

    #[test]
    fn test_initialization_version_negotiation() {
        // Test server accepting client version
        // 测试服务器接受客户端版本
        let client_request = Request::new(
            Method::Initialize,
            Some(json!({
                "protocolVersion": super::super::PROTOCOL_VERSION
            })),
            RequestId::Number(1),
        );

        let server_response = Response::success(
            json!({
                "protocolVersion": super::super::PROTOCOL_VERSION
            }),
            RequestId::Number(1),
        );

        let client_version: String = serde_json::from_value(
            client_request
                .params
                .unwrap()
                .get("protocolVersion")
                .unwrap()
                .clone(),
        )
        .unwrap();

        let server_version: String = serde_json::from_value(
            server_response
                .result
                .unwrap()
                .get("protocolVersion")
                .unwrap()
                .clone(),
        )
        .unwrap();

        // Verify version match
        // 验证版本匹配
        assert_eq!(client_version, server_version);
        assert_eq!(client_version, super::super::PROTOCOL_VERSION);

        // Test server rejecting unsupported version
        // 测试服务器拒绝不支持的版本
        let unsupported_version = "1.0.0";
        let client_request = Request::new(
            Method::Initialize,
            Some(json!({
                "protocolVersion": unsupported_version
            })),
            RequestId::Number(2),
        );

        let server_error = Response::error(
            ResponseError {
                code: error_codes::INVALID_REQUEST,
                message: "Unsupported protocol version".to_string(),
                data: Some(json!({
                    "supported": [super::super::PROTOCOL_VERSION],
                    "requested": unsupported_version
                })),
            },
            RequestId::Number(2),
        );

        // Verify error response format
        // 验证错误响应格式
        let error_json = serde_json::to_string(&server_error).unwrap();
        assert!(error_json.contains("Unsupported protocol version"));
        assert!(error_json.contains(super::super::PROTOCOL_VERSION));
        assert!(error_json.contains(unsupported_version));
    }

    #[test]
    fn test_ping_mechanism() {
        // 测试 ping 请求格式
        // Test ping request format
        let ping_request =
            Request::new(Method::Ping, None, RequestId::String("ping-1".to_string()));

        // 验证请求格式
        // Verify request format
        let request_json = serde_json::to_string(&ping_request).unwrap();
        assert!(request_json.contains(r#""method":"ping""#));
        assert!(request_json.contains(r#""id":"ping-1""#));
        assert!(!request_json.contains("params"));

        // 测试 ping 响应格式
        // Test ping response format
        let ping_response = Response::success(json!({}), RequestId::String("ping-1".to_string()));

        // 验证响应格式
        // Verify response format
        let response_json = serde_json::to_string(&ping_response).unwrap();
        assert!(response_json.contains(r#""result":{}"#));
        assert!(response_json.contains(r#""id":"ping-1""#));
        assert!(!response_json.contains("error"));

        // 测试 ping 请求的 ID 唯一性
        // Test ping request ID uniqueness
        let mut session_ids = HashSet::new();
        assert!(ping_request.validate_id_uniqueness(&mut session_ids));
        assert!(!ping_request.validate_id_uniqueness(&mut session_ids));

        // 测试 ping 响应必须匹配请求 ID
        // Test ping response must match request ID
        let mismatched_response =
            Response::success(json!({}), RequestId::String("wrong-id".to_string()));
        assert_ne!(ping_request.id, mismatched_response.id);

        // 测试 ping 超时错误响应
        // Test ping timeout error response
        let timeout_error = Response::error(
            ResponseError {
                code: error_codes::REQUEST_CANCELLED,
                message: "Ping timeout".to_string(),
                data: None,
            },
            RequestId::String("ping-1".to_string()),
        );

        // 验证错误响应格式
        // Verify error response format
        let error_json = serde_json::to_string(&timeout_error).unwrap();
        assert!(error_json.contains("Ping timeout"));
        assert!(error_json.contains(&error_codes::REQUEST_CANCELLED.to_string()));
    }

    #[test]
    fn test_ping_pong_sequence() {
        // 测试完整的 ping-pong 序列
        // Test complete ping-pong sequence
        let mut session_ids = HashSet::new();

        // 1. 发送 ping 请求
        // 1. Send ping request
        let ping_request = Request::new(
            Method::Ping,
            None,
            RequestId::String("ping-seq-1".to_string()),
        );
        assert!(ping_request.validate_id_uniqueness(&mut session_ids));

        // 2. 接收 pong 响应
        // 2. Receive pong response
        let pong_response =
            Response::success(json!({}), RequestId::String("ping-seq-1".to_string()));

        // 验证响应匹配请求
        // Verify response matches request
        assert_eq!(ping_request.id, pong_response.id);
        assert!(pong_response.result.is_some());
        assert!(pong_response.error.is_none());

        // 3. 测试多个 ping 请求
        // 3. Test multiple ping requests
        let ping_request_2 = Request::new(
            Method::Ping,
            None,
            RequestId::String("ping-seq-2".to_string()),
        );
        assert!(ping_request_2.validate_id_uniqueness(&mut session_ids));

        // 验证不同 ping 请求的 ID 不同
        // Verify different ping requests have different IDs
        assert_ne!(ping_request.id, ping_request_2.id);
    }
}
