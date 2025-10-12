# client/types.rs - 客户端类型定义文档

## 概述

`client/types.rs` 模块定义了与aria2 JSON-RPC通信所需的所有数据结构类型，包括请求、响应、状态信息等。

## JSON-RPC 通信结构

### JsonRpcRequest

JSON-RPC 2.0 请求结构。

#### 字段

- `jsonrpc: String` - JSON-RPC版本号，固定为"2.0"
- `id: String` - 请求的唯一标识符，使用UUID生成
- `method: String` - 要调用的方法名
- `params: Vec<serde_json::Value>` - 方法参数列表

#### 方法

##### new(method: String, params: Vec<serde_json::Value>) -> Self
- **作用**: 创建新的JSON-RPC请求
- **参数**:
  - `method`: RPC方法名
  - `params`: 方法参数
- **返回**: JsonRpcRequest实例
- **特性**: 自动生成UUID作为请求ID

### JsonRpcResponse

JSON-RPC 2.0 响应结构。

#### 字段

- `jsonrpc: String` - JSON-RPC版本号
- `id: String` - 对应请求的ID
- `result: Option<serde_json::Value>` - 成功时的返回结果
- `error: Option<JsonRpcError>` - 错误时的错误信息

### JsonRpcError

JSON-RPC错误信息结构。

#### 字段

- `code: i32` - 错误代码
- `message: String` - 错误消息

## Aria2 特定类型

### Aria2Status

aria2下载任务的状态信息。

#### 字段

- `gid: String` - 任务的全局唯一标识符
- `status: String` - 任务状态（"active", "waiting", "paused", "error", "complete", "removed"）
- `total_length: String` - 总文件大小（字节数，字符串格式）
- `completed_length: String` - 已完成大小（字节数，字符串格式）
- `download_speed: String` - 下载速度（字节/秒，字符串格式）
- `upload_speed: String` - 上传速度（字节/秒，字符串格式）
- `files: Vec<Aria2File>` - 任务包含的文件列表
- `error_code: Option<String>` - 错误代码（可选）
- `error_message: Option<String>` - 错误消息（可选）

#### 特性

- 使用 `#[serde(rename)]` 处理aria2 API的驼峰命名
- 错误相关字段使用 `#[serde(default)]` 标记为可选
- 所有数值都以字符串形式存储（遵循aria2 API设计）

### Aria2File

单个文件的状态信息。

#### 字段

- `index: String` - 文件在任务中的索引
- `path: String` - 文件的完整路径
- `length: String` - 文件总大小（字节数，字符串格式）
- `completed_length: String` - 文件已完成大小（字节数，字符串格式）
- `selected: String` - 文件是否被选中下载（"true"/"false"）

### Aria2Options

下载选项配置结构。

#### 字段

- `dir: String` - 下载目录路径
- `out: Option<String>` - 输出文件名（可选）

#### 特性

- 使用 `#[serde(skip_serializing_if = "Option::is_none")]` 跳过None值的序列化
- 支持aria2的所有基本下载选项

## 测试模块

### 包含的测试用例

#### JSON-RPC 请求测试

1. **test_jsonrpc_request_serialization**
   - 测试JsonRpcRequest的序列化功能
   - 验证必要字段的存在

2. **test_jsonrpc_request_unique_ids**
   - 测试请求ID的唯一性
   - 确保每个请求都有唯一标识符

#### JSON-RPC 响应测试

3. **test_jsonrpc_response_deserialization_success**
   - 测试成功响应的反序列化
   - 验证result字段的正确解析

4. **test_jsonrpc_response_deserialization_error**
   - 测试错误响应的反序列化
   - 验证error字段的正确解析

#### Aria2 状态测试

5. **test_aria2_status_deserialization**
   - 测试Aria2Status的反序列化
   - 验证所有字段的正确映射

6. **test_aria2_status_with_error**
   - 测试包含错误信息的状态反序列化
   - 验证错误字段的处理

7. **test_aria2_status_empty_files**
   - 测试空文件列表的处理
   - 验证默认值行为

#### 文件信息测试

8. **test_aria2_file_deserialization**
   - 测试Aria2File的反序列化
   - 验证文件信息字段的正确解析

#### 选项配置测试

9. **test_aria2_options_serialization**
   - 测试包含所有字段的选项序列化
   - 验证字段的正确输出

10. **test_aria2_options_serialization_without_out**
    - 测试跳过None字段的序列化行为
    - 验证条件序列化功能

## 设计特点

1. **类型安全**: 使用强类型结构体确保数据的正确性
2. **序列化支持**: 完整的serde支持，自动处理JSON转换
3. **字段映射**: 处理aria2 API的命名约定差异
4. **可选字段**: 正确处理可选和默认字段
5. **测试覆盖**: 全面的单元测试确保类型定义的正确性

## 使用示例

```rust
use crate::client::types::*;

// 创建RPC请求
let request = JsonRpcRequest::new(
    "aria2.addUri".to_string(),
    vec![serde_json::json!(["http://example.com/file.bin"])]
);

// 配置下载选项
let options = Aria2Options {
    dir: "/downloads".to_string(),
    out: Some("custom_name.bin".to_string()),
};

// 解析状态响应
let status_json = r#"{
    "gid": "2089b05ecca3d829",
    "status": "active",
    "totalLength": "1048576",
    "completedLength": "524288",
    "downloadSpeed": "102400",
    "uploadSpeed": "0",
    "files": []
}"#;

let status: Aria2Status = serde_json::from_str(status_json)?;
println!("任务 {} 状态: {}", status.gid, status.status);
```

## 依赖关系

- `serde`: 序列化/反序列化框架
- `uuid`: UUID生成（用于请求ID）
- `serde_json`: JSON处理