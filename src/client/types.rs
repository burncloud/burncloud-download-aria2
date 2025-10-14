use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: Vec<serde_json::Value>,
}

impl JsonRpcRequest {
    pub fn new(method: String, params: Vec<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            method,
            params,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: String,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

// Aria2Status and Aria2File have been removed
// All aria2 data is now accessed directly from JSON responses for real-time data

#[derive(Debug, Clone, Serialize)]
pub struct Aria2Options {
    pub dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest::new(
            "aria2.addUri".to_string(),
            vec![serde_json::json!(["http://example.com/file.bin"])]
        );

        let serialized = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"method\":\"aria2.addUri\""));
        assert!(serialized.contains("\"id\":"));
        assert!(serialized.contains("\"params\""));
    }

    #[test]
    fn test_jsonrpc_response_deserialization_success() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": "test-id",
            "result": "2089b05ecca3d829"
        }"#;

        let response: JsonRpcResponse = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, "test-id");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_jsonrpc_response_deserialization_error() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": "test-id",
            "error": {
                "code": 1,
                "message": "Unauthorized"
            }
        }"#;

        let response: JsonRpcResponse = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, 1);
        assert_eq!(error.message, "Unauthorized");
    }

    #[test]
    fn test_aria2_options_serialization() {
        let options = Aria2Options {
            dir: "/downloads".to_string(),
            out: Some("output.bin".to_string()),
        };

        let serialized = serde_json::to_string(&options).expect("Failed to serialize");
        assert!(serialized.contains("\"dir\":\"/downloads\""));
        assert!(serialized.contains("\"out\":\"output.bin\""));
    }

    #[test]
    fn test_aria2_options_serialization_without_out() {
        let options = Aria2Options {
            dir: "/downloads".to_string(),
            out: None,
        };

        let serialized = serde_json::to_string(&options).expect("Failed to serialize");
        assert!(serialized.contains("\"dir\":\"/downloads\""));
        assert!(!serialized.contains("\"out\""));
    }

    #[test]
    fn test_jsonrpc_request_unique_ids() {
        let request1 = JsonRpcRequest::new("method1".to_string(), vec![]);
        let request2 = JsonRpcRequest::new("method2".to_string(), vec![]);

        // IDs should be unique
        assert_ne!(request1.id, request2.id);
    }
}