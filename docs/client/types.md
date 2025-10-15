# 客户端数据类型 (client/types.rs)

## 概述

`types.rs` 模块定义了客户端与 Aria2 进行 JSON-RPC 通信所需的所有数据结构。这些类型负责序列化和反序列化 JSON-RPC 协议数据，确保与 Aria2 的兼容性。

## 核心数据类型

### JSON-RPC 协议类型

#### `JsonRpcRequest`

```rust
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,    // 协议版本，固定为 "2.0"
    pub id: String,         // 请求唯一标识符
    pub method: String,     // 调用的方法名
    pub params: Vec<serde_json::Value>,  // 方法参数数组
}
```

**功能**: 表示 JSON-RPC 2.0 标准的请求对象

**字段说明**:
- `jsonrpc`: 协议版本标识，始终为 "2.0"
- `id`: 请求的唯一标识符，使用 UUID v4 生成
- `method`: 要调用的 Aria2 方法名（如 "aria2.addUri"）
- `params`: 方法参数，第一个参数通常是认证令牌

**构造方法**:
```rust
impl JsonRpcRequest {
    pub fn new(method: String, params: Vec<serde_json::Value>) -> Self
}
```

**使用示例**:
```rust
let request = JsonRpcRequest::new(
    "aria2.addUri".to_string(),
    vec![
        json!("token:burncloud"),           // 认证令牌
        json!(["https://example.com/file.zip"]),  // URI 列表
        json!({"dir": "/downloads"})        // 选项
    ]
);
```

**序列化结果**:
```json
{
    "jsonrpc": "2.0",
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "method": "aria2.addUri",
    "params": [
        "token:burncloud",
        ["https://example.com/file.zip"],
        {"dir": "/downloads"}
    ]
}
```

#### `JsonRpcResponse`

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,                    // 协议版本
    pub id: String,                         // 对应请求的ID
    #[serde(default)]
    pub result: Option<serde_json::Value>,  // 成功时的结果
    #[serde(default)]
    pub error: Option<JsonRpcError>,        // 错误时的错误信息
}
```

**功能**: 表示 JSON-RPC 2.0 标准的响应对象

**字段说明**:
- `jsonrpc`: 协议版本，应为 "2.0"
- `id`: 与请求对应的标识符
- `result`: 成功响应的结果数据（与 `error` 互斥）
- `error`: 错误响应的错误信息（与 `result` 互斥）

**成功响应示例**:
```json
{
    "jsonrpc": "2.0",
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "result": "2089b05ecca3d829"
}
```

**错误响应示例**:
```json
{
    "jsonrpc": "2.0",
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "error": {
        "code": 1,
        "message": "Unauthorized"
    }
}
```

#### `JsonRpcError`

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,      // 错误代码
    pub message: String, // 错误消息
}
```

**功能**: 表示 JSON-RPC 错误信息

**常见错误代码**:
- `1`: 认证失败
- `2`: 方法不存在
- `3`: 参数错误
- `6`: GID 不存在

### Aria2 配置类型

#### `Aria2Options`

```rust
#[derive(Debug, Clone, Serialize)]
pub struct Aria2Options {
    pub dir: String,                               // 下载目录
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,                       // 输出文件名
}
```

**功能**: Aria2 下载任务的配置选项

**字段说明**:
- `dir`: 下载文件保存的目录路径
- `out`: 可选的输出文件名，如果不指定则使用 URL 中的文件名

**序列化特性**:
- `out` 字段在为 `None` 时不会被序列化，减少不必要的参数传递

**使用示例**:
```rust
// 指定输出文件名
let options = Aria2Options {
    dir: "/home/user/downloads".to_string(),
    out: Some("renamed_file.zip".to_string()),
};

// 使用默认文件名
let options = Aria2Options {
    dir: "/home/user/downloads".to_string(),
    out: None,
};
```

**序列化结果**:
```json
// 有输出文件名
{
    "dir": "/home/user/downloads",
    "out": "renamed_file.zip"
}

// 无输出文件名
{
    "dir": "/home/user/downloads"
}
```

## 设计原则

### 实时数据访问

项目采用直接访问 JSON 响应数据的设计，而不是预定义的结构体：

**原因**:
1. **灵活性**: Aria2 返回的状态信息字段很多，预定义结构体容易遗漏
2. **实时性**: 避免数据转换的性能开销
3. **兼容性**: 不同版本的 Aria2 可能返回不同的字段

**实现方式**:
```rust
// 直接访问 JSON 字段
let status = client.tell_status("gid").await?;
let progress = status["completedLength"].as_str().unwrap_or("0");
let total = status["totalLength"].as_str().unwrap_or("0");
let speed = status["downloadSpeed"].as_str().unwrap_or("0");
```

### 序列化优化

使用 `serde` 的高级特性优化序列化：

1. **条件序列化**: `skip_serializing_if` 避免空值传递
2. **默认值**: `#[serde(default)]` 处理可选字段
3. **字段重命名**: 保持与 Aria2 API 的兼容性

### 唯一标识符

使用 UUID v4 生成请求 ID：
- **唯一性**: 避免 ID 冲突
- **随机性**: 提高安全性
- **标准化**: 遵循 RFC 4122 标准

## 测试用例

模块包含完整的单元测试，验证：

### 序列化测试
```rust
#[test]
fn test_jsonrpc_request_serialization() {
    let request = JsonRpcRequest::new(
        "aria2.addUri".to_string(),
        vec![serde_json::json!(["http://example.com/file.bin"])]
    );

    let serialized = serde_json::to_string(&request).expect("Failed to serialize");
    assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
    assert!(serialized.contains("\"method\":\"aria2.addUri\""));
}
```

### 反序列化测试
```rust
#[test]
fn test_jsonrpc_response_deserialization_success() {
    let json = r#"{
        "jsonrpc": "2.0",
        "id": "test-id",
        "result": "2089b05ecca3d829"
    }"#;

    let response: JsonRpcResponse = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
}
```

### 唯一性测试
```rust
#[test]
fn test_jsonrpc_request_unique_ids() {
    let request1 = JsonRpcRequest::new("method1".to_string(), vec![]);
    let request2 = JsonRpcRequest::new("method2".to_string(), vec![]);

    // ID 应该是唯一的
    assert_ne!(request1.id, request2.id);
}
```

## 扩展性

### 添加新选项

如需添加更多 Aria2 选项，可以扩展 `Aria2Options`：

```rust
#[derive(Debug, Clone, Serialize)]
pub struct Aria2Options {
    pub dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,

    // 新增选项
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_connection_per_server: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<u32>,
}
```

### 自定义序列化

可以为特殊需求实现自定义序列化：

```rust
impl Serialize for CustomType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 自定义序列化逻辑
    }
}
```

## 相关文档

- [客户端主模块](./mod.md) - Aria2Client 的详细实现
- [错误处理](../error.md) - 错误类型和处理策略
- [JSON-RPC 2.0 规范](https://www.jsonrpc.org/specification) - 协议标准