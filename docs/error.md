# 错误处理 (error.rs)

## 概述

`error.rs` 模块定义了 BurnCloud Aria2 Download Manager 中所有可能出现的错误类型，使用 `thiserror` crate 提供统一的错误处理。

## 错误类型

### 通信错误

#### `RpcError(i32, String)`
- **描述**: JSON-RPC 协议错误
- **参数**:
  - `i32`: 错误代码
  - `String`: 错误消息
- **触发场景**:
  - Aria2 RPC服务返回错误响应
  - RPC协议层面的问题

#### `TransportError(reqwest::Error)`
- **描述**: HTTP传输层错误
- **来源**: `reqwest::Error`
- **触发场景**:
  - 网络连接问题
  - HTTP请求失败
  - 超时等传输层问题

#### `SerializationError(serde_json::Error)`
- **描述**: JSON序列化/反序列化错误
- **来源**: `serde_json::Error`
- **触发场景**:
  - JSON格式错误
  - 数据结构不匹配

### 守护进程相关错误

#### `DaemonUnavailable(String)`
- **描述**: Aria2 守护进程不可用
- **触发场景**:
  - 守护进程未启动
  - 连接失败
  - RPC服务不响应

#### `BinaryDownloadFailed(String)`
- **描述**: 二进制文件下载失败
- **触发场景**:
  - Aria2二进制文件下载失败
  - 网络问题导致下载中断

#### `BinaryExtractionFailed(String)`
- **描述**: 二进制文件解压失败
- **触发场景**:
  - 压缩包损坏
  - 解压过程中的IO错误
  - 权限问题

### 进程管理错误

#### `ProcessStartFailed(String)`
- **描述**: 进程启动失败
- **触发场景**:
  - 二进制文件不存在或损坏
  - 权限不足
  - 系统资源不足

#### `ProcessManagementError(String)`
- **描述**: 进程管理错误
- **触发场景**:
  - 进程监控失败
  - 进程状态查询错误
  - 进程停止失败

#### `RestartLimitExceeded`
- **描述**: 超过最大重启次数限制
- **触发场景**:
  - 守护进程反复崩溃
  - 达到配置的最大重启次数

### 任务相关错误

#### `TaskNotFound(String)`
- **描述**: 任务未找到
- **触发场景**:
  - TaskId无效
  - 任务已被删除
  - GID映射丢失

#### `InvalidUrl(String)`
- **描述**: 无效的URL格式
- **触发场景**:
  - URL格式错误
  - 不支持的协议

#### `InvalidPath(String)`
- **描述**: 无效的文件路径
- **触发场景**:
  - 路径格式错误
  - 路径不存在或无权限访问

#### `UnsupportedType(String)`
- **描述**: 不支持的下载类型
- **触发场景**:
  - 未知的文件类型
  - 不支持的协议

### 状态管理错误

#### `StateMappingError(String)`
- **描述**: 状态映射错误
- **触发场景**:
  - Aria2状态与内部状态映射失败
  - 状态转换错误

### 系统级错误

#### `IoError(std::io::Error)`
- **描述**: IO操作错误
- **来源**: `std::io::Error`
- **触发场景**:
  - 文件读写错误
  - 目录创建失败
  - 权限问题

#### `General(String)`
- **描述**: 通用错误
- **触发场景**:
  - 其他未分类的错误情况
  - 内部逻辑错误

## 错误处理策略

### 自动重试机制
- `ProcessStartFailed`: 自动重启进程（有重试次数限制）
- `DaemonUnavailable`: 健康检查器会尝试重新连接

### 优雅降级
- `BinaryDownloadFailed`: 提示用户手动下载二进制文件
- `TransportError`: 重试网络请求

### 用户反馈
- 所有错误都提供详细的错误消息
- Debug模式下提供更多调试信息

## 使用示例

```rust
use crate::error::Aria2Error;

// 处理RPC错误
match client.tell_status("gid").await {
    Ok(status) => { /* 处理状态 */ },
    Err(Aria2Error::RpcError(code, msg)) => {
        eprintln!("RPC错误 {}: {}", code, msg);
    },
    Err(Aria2Error::DaemonUnavailable(msg)) => {
        eprintln!("守护进程不可用: {}", msg);
        // 尝试重启守护进程
    },
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

## 错误传播

项目中错误传播使用以下模式：

1. **内部错误**: 使用 `?` 操作符自动转换
2. **跨层错误**: 通过 `From` trait 实现自动转换
3. **用户接口**: 返回 `anyhow::Result` 为用户提供灵活的错误处理

```rust
// 自动错误转换示例
pub async fn download_binary() -> Result<(), Aria2Error> {
    let response = reqwest::get("https://example.com/aria2c").await?; // TransportError
    let bytes = response.bytes().await?; // TransportError
    std::fs::write("/path/to/aria2c", bytes)?; // IoError
    Ok(())
}
```