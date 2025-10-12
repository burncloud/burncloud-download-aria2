# client/mod.rs - Aria2客户端模块文档

## 概述

`client/mod.rs` 模块提供了与aria2 JSON-RPC服务通信的核心客户端实现。它包含了所有与aria2交互所需的方法和功能。

## 主要结构

### Aria2Client

Aria2 JSON-RPC客户端的主要实现结构。

#### 字段

- `rpc_url: String` - aria2 RPC服务的URL地址
- `secret: Option<String>` - 可选的认证密钥
- `http_client: reqwest::Client` - HTTP客户端实例

#### 构造函数

##### new(rpc_url: String, secret: Option<String>) -> Self
- **作用**: 创建新的Aria2Client实例
- **参数**:
  - `rpc_url`: aria2 RPC服务地址
  - `secret`: 可选的认证密钥
- **返回**: Aria2Client实例

## 私有方法

### call_rpc(&self, method: String, params: Vec<serde_json::Value>) -> Result<serde_json::Value, Aria2Error>
- **作用**: 执行JSON-RPC调用的核心方法
- **参数**:
  - `method`: RPC方法名
  - `params`: 方法参数列表
- **返回**: RPC调用结果或错误
- **功能**:
  - 自动添加认证token（如果配置了secret）
  - 处理HTTP请求和响应
  - 解析JSON-RPC响应
  - 错误处理和转换

## 公共方法

### 下载添加方法

#### add_uri(&self, uris: Vec<String>, options: Aria2Options) -> Result<String, Aria2Error>
- **作用**: 添加URI下载任务（HTTP/HTTPS/FTP/Magnet）
- **参数**:
  - `uris`: 下载链接列表
  - `options`: 下载选项配置
- **返回**: 任务GID或错误
- **RPC方法**: `aria2.addUri`

#### add_torrent(&self, torrent_data: Vec<u8>, options: Aria2Options) -> Result<String, Aria2Error>
- **作用**: 添加种子文件下载任务
- **参数**:
  - `torrent_data`: 种子文件的二进制数据
  - `options`: 下载选项配置
- **返回**: 任务GID或错误
- **RPC方法**: `aria2.addTorrent`
- **特性**: 自动进行Base64编码

#### add_metalink(&self, metalink_data: Vec<u8>, options: Aria2Options) -> Result<String, Aria2Error>
- **作用**: 添加Metalink文件下载任务
- **参数**:
  - `metalink_data`: Metalink文件的二进制数据
  - `options`: 下载选项配置
- **返回**: 任务GID或错误
- **RPC方法**: `aria2.addMetalink`
- **特性**: 自动进行Base64编码

### 状态查询方法

#### tell_status(&self, gid: &str) -> Result<Aria2Status, Aria2Error>
- **作用**: 获取指定任务的详细状态信息
- **参数**:
  - `gid`: 任务的唯一标识符
- **返回**: 任务状态结构或错误
- **RPC方法**: `aria2.tellStatus`

#### tell_active(&self) -> Result<Vec<Aria2Status>, Aria2Error>
- **作用**: 获取所有活动下载任务的状态列表
- **返回**: 活动任务状态列表或错误
- **RPC方法**: `aria2.tellActive`

#### tell_stopped(&self, offset: i32, num: i32) -> Result<Vec<Aria2Status>, Aria2Error>
- **作用**: 获取已停止的下载任务列表
- **参数**:
  - `offset`: 起始偏移量
  - `num`: 返回数量
- **返回**: 已停止任务状态列表或错误
- **RPC方法**: `aria2.tellStopped`

#### tell_waiting(&self, offset: i32, num: i32) -> Result<Vec<Aria2Status>, Aria2Error>
- **作用**: 获取等待中的下载任务列表
- **参数**:
  - `offset`: 起始偏移量
  - `num`: 返回数量
- **返回**: 等待任务状态列表或错误
- **RPC方法**: `aria2.tellWaiting`

### 任务控制方法

#### pause(&self, gid: &str) -> Result<(), Aria2Error>
- **作用**: 暂停指定的下载任务
- **参数**:
  - `gid`: 任务的唯一标识符
- **返回**: 成功或错误
- **RPC方法**: `aria2.pause`

#### unpause(&self, gid: &str) -> Result<(), Aria2Error>
- **作用**: 恢复指定的下载任务
- **参数**:
  - `gid`: 任务的唯一标识符
- **返回**: 成功或错误
- **RPC方法**: `aria2.unpause`

#### remove(&self, gid: &str) -> Result<(), Aria2Error>
- **作用**: 移除指定的下载任务
- **参数**:
  - `gid`: 任务的唯一标识符
- **返回**: 成功或错误
- **RPC方法**: `aria2.remove`

### 全局状态方法

#### get_global_stat(&self) -> Result<serde_json::Value, Aria2Error>
- **作用**: 获取aria2的全局统计信息
- **返回**: 全局统计数据或错误
- **RPC方法**: `aria2.getGlobalStat`

## 特性和功能

1. **自动认证**: 如果配置了secret，会自动在所有RPC调用中添加token
2. **错误处理**: 完整的错误处理和转换机制
3. **异步支持**: 所有方法都是异步的，支持并发操作
4. **类型安全**: 使用强类型结构体进行数据交换
5. **Base64编码**: 自动处理二进制数据的编码

## 使用示例

```rust
use crate::client::Aria2Client;
use crate::client::types::Aria2Options;

// 创建客户端
let client = Aria2Client::new(
    "http://localhost:6800/jsonrpc".to_string(),
    Some("my_secret".to_string())
);

// 添加HTTP下载
let options = Aria2Options {
    dir: "/downloads".to_string(),
    out: Some("file.zip".to_string()),
};

let gid = client.add_uri(
    vec!["http://example.com/file.zip".to_string()],
    options
).await?;

// 查询状态
let status = client.tell_status(&gid).await?;
println!("下载进度: {}/{}", status.completed_length, status.total_length);

// 控制下载
client.pause(&gid).await?;  // 暂停
client.unpause(&gid).await?; // 恢复
client.remove(&gid).await?;  // 移除
```

## 依赖关系

- `reqwest`: HTTP客户端
- `serde_json`: JSON序列化/反序列化
- `base64`: Base64编码
- `crate::error::Aria2Error`: 错误类型
- `types::*`: 类型定义