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

// Aria2 specific response types
#[derive(Debug, Clone, Deserialize)]
pub struct Aria2Status {
    pub gid: String,
    pub status: String,
    #[serde(rename = "totalLength")]
    pub total_length: String,
    #[serde(rename = "completedLength")]
    pub completed_length: String,
    #[serde(rename = "downloadSpeed")]
    pub download_speed: String,
    #[serde(rename = "uploadSpeed")]
    pub upload_speed: String,
    #[serde(default)]
    pub files: Vec<Aria2File>,
    #[serde(rename = "errorCode", default)]
    pub error_code: Option<String>,
    #[serde(rename = "errorMessage", default)]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Aria2File {
    pub index: String,
    pub path: String,
    pub length: String,
    #[serde(rename = "completedLength")]
    pub completed_length: String,
    pub selected: String,
}

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
    fn test_aria2_status_deserialization() {
        let json = r#"{
            "gid": "2089b05ecca3d829",
            "status": "active",
            "totalLength": "1048576",
            "completedLength": "524288",
            "downloadSpeed": "102400",
            "uploadSpeed": "0",
            "files": [
                {
                    "index": "1",
                    "path": "/downloads/file.bin",
                    "length": "1048576",
                    "completedLength": "524288",
                    "selected": "true"
                }
            ]
        }"#;

        let status: Aria2Status = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(status.gid, "2089b05ecca3d829");
        assert_eq!(status.status, "active");
        assert_eq!(status.total_length, "1048576");
        assert_eq!(status.completed_length, "524288");
        assert_eq!(status.download_speed, "102400");
        assert_eq!(status.files.len(), 1);
        assert!(status.error_code.is_none());
    }

    #[test]
    fn test_aria2_status_with_error() {
        let json = r#"{
            "gid": "2089b05ecca3d829",
            "status": "error",
            "totalLength": "0",
            "completedLength": "0",
            "downloadSpeed": "0",
            "uploadSpeed": "0",
            "files": [],
            "errorCode": "1",
            "errorMessage": "Network error"
        }"#;

        let status: Aria2Status = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(status.status, "error");
        assert_eq!(status.error_code, Some("1".to_string()));
        assert_eq!(status.error_message, Some("Network error".to_string()));
    }

    #[test]
    fn test_aria2_file_deserialization() {
        let json = r#"{
            "index": "1",
            "path": "/downloads/file.bin",
            "length": "1048576",
            "completedLength": "524288",
            "selected": "true"
        }"#;

        let file: Aria2File = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(file.index, "1");
        assert_eq!(file.path, "/downloads/file.bin");
        assert_eq!(file.length, "1048576");
        assert_eq!(file.completed_length, "524288");
        assert_eq!(file.selected, "true");
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
    fn test_aria2_status_empty_files() {
        let json = r#"{
            "gid": "test",
            "status": "waiting",
            "totalLength": "0",
            "completedLength": "0",
            "downloadSpeed": "0",
            "uploadSpeed": "0",
            "files": []
        }"#;

        let status: Aria2Status = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(status.files.len(), 0);
    }

    #[test]
    fn test_jsonrpc_request_unique_ids() {
        let request1 = JsonRpcRequest::new("method1".to_string(), vec![]);
        let request2 = JsonRpcRequest::new("method2".to_string(), vec![]);

        // IDs should be unique
        assert_ne!(request1.id, request2.id);
    }
}