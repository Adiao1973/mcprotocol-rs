use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

use super::{ClientCapabilities, ImplementationInfo, RequestId, ServerCapabilities};
use crate::Result;

/// Base JSON-RPC message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Response(Response),
    Notification(Notification),
}

/// JSON-RPC request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Protocol version (must be "2.0")
    pub jsonrpc: String,
    /// Request method
    pub method: String,
    /// Optional parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Request ID
    pub id: RequestId,
}

/// JSON-RPC response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Protocol version (must be "2.0")
    pub jsonrpc: String,
    /// Request ID
    pub id: RequestId,
    /// Response result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

/// JSON-RPC notification message (request without ID)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Protocol version (must be "2.0")
    pub jsonrpc: String,
    /// Notification method
    pub method: String,
    /// Optional parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard error codes
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Method {
    // Lifecycle methods
    Initialize,
    Initialized,
    Shutdown,
    Exit,

    // Utility methods
    #[serde(rename = "$/cancelRequest")]
    Cancel,
    #[serde(rename = "$/ping")]
    Ping,
    #[serde(rename = "$/pong")]
    Pong,
    #[serde(rename = "$/progress")]
    Progress,

    // Server feature methods
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
    #[serde(rename = "roots/list")]
    ListRoots,
    #[serde(rename = "roots/get")]
    GetRoot,

    #[serde(rename = "sampling/request")]
    SamplingRequest,
}

impl Request {
    /// Creates a new request
    pub fn new(method: Method, params: Option<Value>, id: RequestId) -> Self {
        Self {
            jsonrpc: super::JSONRPC_VERSION.to_string(),
            method: method.to_string(),
            params,
            id,
        }
    }
}

impl Response {
    /// Creates a new successful response
    pub fn success(result: Value, id: RequestId) -> Self {
        Self {
            jsonrpc: super::JSONRPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Creates a new error response
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
