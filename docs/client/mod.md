# 客户端模块 (client/)

## 概述

客户端模块提供与 Aria2 守护进程进行 JSON-RPC 通信的功能。该模块封装了所有的网络通信细节，为上层模块提供简洁的异步接口。

## 模块结构

- **mod.rs** - Aria2Client 主要实现
- **types.rs** - 客户端相关的数据类型定义

## 核心组件

### Aria2Client

`Aria2Client` 是与 Aria2 守护进程通信的核心客户端类。

#### 结构定义
```rust
pub struct Aria2Client {
    rpc_url: String,
    secret: Option<String>,
    http_client: reqwest::Client,
}
```

**字段说明**:
- `rpc_url`: Aria2 JSON-RPC 端点地址
- `secret`: 可选的 RPC 认证令牌
- `http_client`: HTTP 客户端实例

## 主要功能

### 1. 客户端创建

#### `new(rpc_url: String, secret: Option<String>) -> Self`
- **功能**: 创建新的 Aria2Client 实例
- **参数**:
  - `rpc_url`: RPC 服务地址
  - `secret`: 可选的认证令牌
- **返回**: 客户端实例

### 2. 下载任务管理

#### `add_uri(&self, uris: Vec<String>, options: Aria2Options) -> Result<String, Aria2Error>`
- **功能**: 添加 URI 下载任务（HTTP/HTTPS/FTP/Magnet）
- **参数**:
  - `uris`: 下载链接列表
  - `options`: 下载选项配置
- **返回**: 任务的 GID（全局标识符）
- **用途**:
  - HTTP/HTTPS 文件下载
  - FTP 文件下载
  - Magnet 链接下载

**示例**:
```rust
let gid = client.add_uri(
    vec!["https://example.com/file.zip".to_string()],
    Aria2Options {
        dir: "/downloads".to_string(),
        out: Some("file.zip".to_string()),
    }
).await?;
```

#### `add_torrent(&self, torrent_data: Vec<u8>, options: Aria2Options) -> Result<String, Aria2Error>`
- **功能**: 添加 BitTorrent 下载任务
- **参数**:
  - `torrent_data`: .torrent 文件的二进制数据
  - `options`: 下载选项配置
- **返回**: 任务的 GID
- **特性**: 自动 Base64 编码种子文件数据

**示例**:
```rust
let torrent_bytes = std::fs::read("movie.torrent")?;
let gid = client.add_torrent(torrent_bytes, options).await?;
```

#### `add_metalink(&self, metalink_data: Vec<u8>, options: Aria2Options) -> Result<String, Aria2Error>`
- **功能**: 添加 Metalink 下载任务
- **参数**:
  - `metalink_data`: .metalink 文件的二进制数据
  - `options`: 下载选项配置
- **返回**: 任务的 GID
- **用途**: 多源下载，支持备用链接

### 3. 任务控制

#### `pause(&self, gid: &str) -> Result<(), Aria2Error>`
- **功能**: 暂停指定的下载任务
- **参数**: `gid` - 任务的全局标识符
- **用途**: 临时暂停下载，可随时恢复

#### `unpause(&self, gid: &str) -> Result<(), Aria2Error>`
- **功能**: 恢复指定的暂停下载任务
- **参数**: `gid` - 任务的全局标识符
- **用途**: 恢复之前暂停的下载

#### `remove(&self, gid: &str) -> Result<(), Aria2Error>`
- **功能**: 移除指定的下载任务
- **参数**: `gid` - 任务的全局标识符
- **用途**: 彻底删除任务，无法恢复

### 4. 状态查询

#### `tell_status(&self, gid: &str) -> Result<serde_json::Value, Aria2Error>`
- **功能**: 获取指定任务的状态信息
- **参数**: `gid` - 任务的全局标识符
- **返回**: 原始 JSON 状态数据
- **用途**: 实时获取下载进度、速度等信息

**返回的 JSON 包含**:
- `totalLength`: 文件总大小
- `completedLength`: 已下载大小
- `downloadSpeed`: 当前下载速度
- `status`: 任务状态（active/waiting/paused/error/complete/removed）
- `files`: 文件信息数组

#### `tell_active() -> Result<serde_json::Value, Aria2Error>`
- **功能**: 获取所有活跃下载任务
- **返回**: 包含所有活跃任务的 JSON 数组
- **用途**: 获取当前正在下载的任务列表

#### `tell_waiting(offset: i32, num: i32) -> Result<serde_json::Value, Aria2Error>`
- **功能**: 获取等待中的下载任务
- **参数**:
  - `offset`: 起始偏移量
  - `num`: 返回任务数量
- **返回**: 等待任务的 JSON 数组

#### `tell_stopped(offset: i32, num: i32) -> Result<serde_json::Value, Aria2Error>`
- **功能**: 获取已停止的下载任务
- **参数**:
  - `offset`: 起始偏移量
  - `num`: 返回任务数量
- **返回**: 已停止任务的 JSON 数组

### 5. 全局信息

#### `get_global_stat() -> Result<serde_json::Value, Aria2Error>`
- **功能**: 获取 Aria2 全局统计信息
- **返回**: 包含全局统计的 JSON 对象
- **包含信息**:
  - 当前下载速度
  - 活跃下载数量
  - 等待下载数量
  - 已停止下载数量

## 内部实现

### RPC 调用机制

#### `call_rpc(&self, method: String, params: Vec<serde_json::Value>) -> Result<serde_json::Value, Aria2Error>`

**功能**: 执行底层的 JSON-RPC 调用

**实现细节**:
1. **认证处理**: 如果配置了 secret，自动在参数前添加 `token:secret`
2. **请求构建**: 创建符合 JSON-RPC 2.0 标准的请求
3. **HTTP 传输**: 使用 reqwest 发送 POST 请求
4. **响应解析**: 解析 JSON-RPC 响应，处理错误情况
5. **错误转换**: 将 RPC 错误转换为 Aria2Error

**请求格式**:
```json
{
    "jsonrpc": "2.0",
    "id": "1",
    "method": "aria2.addUri",
    "params": ["token:secret", ["http://example.com/file"], {"dir": "/downloads"}]
}
```

**响应格式**:
```json
{
    "jsonrpc": "2.0",
    "id": "1",
    "result": "2089b05ecca3d829"
}
```

### 错误处理

客户端处理多种错误情况：

1. **网络错误**: 连接失败、超时等
2. **RPC 错误**: Aria2 返回的业务错误
3. **序列化错误**: JSON 数据格式问题
4. **认证错误**: 令牌验证失败

### 异步设计

所有公共方法都是异步的，使用 `async/await` 语法：
- 非阻塞网络 I/O
- 支持并发调用
- 与 Tokio 运行时集成

## 使用示例

### 基本使用流程

```rust
use crate::client::{Aria2Client, types::Aria2Options};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建客户端
    let client = Aria2Client::new(
        "http://localhost:6800/jsonrpc".to_string(),
        Some("burncloud".to_string())
    );

    // 2. 添加下载任务
    let options = Aria2Options {
        dir: "/home/user/downloads".to_string(),
        out: Some("myfile.zip".to_string()),
    };

    let gid = client.add_uri(
        vec!["https://example.com/file.zip".to_string()],
        options
    ).await?;

    println!("任务已添加，GID: {}", gid);

    // 3. 查询任务状态
    loop {
        let status = client.tell_status(&gid).await?;

        let total = status["totalLength"].as_str().unwrap_or("0");
        let completed = status["completedLength"].as_str().unwrap_or("0");
        let speed = status["downloadSpeed"].as_str().unwrap_or("0");

        println!("进度: {}/{}, 速度: {} B/s", completed, total, speed);

        if status["status"].as_str() == Some("complete") {
            println!("下载完成！");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
```

### 错误处理示例

```rust
match client.add_uri(uris, options).await {
    Ok(gid) => println!("任务创建成功: {}", gid),
    Err(Aria2Error::DaemonUnavailable(msg)) => {
        eprintln!("Aria2 守护进程不可用: {}", msg);
        // 尝试重启守护进程
    },
    Err(Aria2Error::RpcError(code, msg)) => {
        eprintln!("RPC 错误 {}: {}", code, msg);
        // 根据错误代码处理特定问题
    },
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

## 相关文档

- [数据类型](./types.md) - 客户端相关的数据结构定义
- [错误处理](../error.md) - 详细的错误类型说明
- [管理器模块](../manager/) - 使用客户端的上层封装