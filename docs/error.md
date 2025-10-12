# error.rs - 错误处理模块文档

## 概述

`error.rs` 模块定义了 BurnCloud Aria2 Download Manager 中使用的所有错误类型。使用 `thiserror` crate 提供了结构化的错误处理。

## 错误类型

### Aria2Error

主要的错误枚举类型，包含了所有可能在aria2下载管理过程中出现的错误情况。

#### 错误变体

##### 1. RpcError(i32, String)
- **作用**: JSON-RPC通信错误
- **参数**:
  - `i32`: 错误代码
  - `String`: 错误消息
- **使用场景**: 与aria2 RPC服务通信时出现的错误

##### 2. TransportError(reqwest::Error)
- **作用**: HTTP传输错误
- **自动转换**: 使用 `#[from]` 属性从 `reqwest::Error` 自动转换
- **使用场景**: 网络请求失败、连接超时等HTTP相关错误

##### 3. SerializationError(serde_json::Error)
- **作用**: JSON序列化/反序列化错误
- **自动转换**: 使用 `#[from]` 属性从 `serde_json::Error` 自动转换
- **使用场景**: JSON数据解析失败、序列化失败等

##### 4. DaemonUnavailable(String)
- **作用**: Aria2守护进程不可用
- **参数**: `String` - 详细错误信息
- **使用场景**: aria2守护进程未运行或无法连接时

##### 5. BinaryDownloadFailed(String)
- **作用**: 二进制文件下载失败
- **参数**: `String` - 失败原因
- **使用场景**: 下载aria2二进制文件失败时

##### 6. BinaryExtractionFailed(String)
- **作用**: 二进制文件解压失败
- **参数**: `String` - 失败原因
- **使用场景**: 解压下载的aria2压缩包失败时

##### 7. ProcessStartFailed(String)
- **作用**: 进程启动失败
- **参数**: `String` - 失败原因
- **使用场景**: 启动aria2进程失败时

##### 8. ProcessManagementError(String)
- **作用**: 进程管理错误
- **参数**: `String` - 错误详情
- **使用场景**: 进程监控、重启等管理操作失败时

##### 9. RestartLimitExceeded
- **作用**: 超过最大重启尝试次数
- **使用场景**: aria2进程重启次数超过预设限制时

##### 10. IoError(std::io::Error)
- **作用**: 输入/输出错误
- **自动转换**: 使用 `#[from]` 属性从 `std::io::Error` 自动转换
- **使用场景**: 文件操作、网络IO等系统级错误

##### 11. TaskNotFound(String)
- **作用**: 在aria2中找不到指定任务
- **参数**: `String` - 任务ID或相关信息
- **使用场景**: 查询、操作不存在的下载任务时

##### 12. InvalidUrl(String)
- **作用**: 无效的URL格式
- **参数**: `String` - 无效的URL
- **使用场景**: URL格式验证失败时

##### 13. InvalidPath(String)
- **作用**: 无效的文件路径
- **参数**: `String` - 无效的路径
- **使用场景**: 文件路径验证失败时

##### 14. UnsupportedType(String)
- **作用**: 不支持的下载类型
- **参数**: `String` - 不支持的类型描述
- **使用场景**: 尝试下载不支持的文件类型时

##### 15. StateMappingError(String)
- **作用**: 状态映射错误
- **参数**: `String` - 映射错误详情
- **使用场景**: aria2状态与内部状态转换失败时

##### 16. General(String)
- **作用**: 通用错误
- **参数**: `String` - 错误描述
- **使用场景**: 其他未分类的错误情况

## 错误处理特性

1. **自动转换**: 通过 `#[from]` 属性实现从标准库错误类型的自动转换
2. **格式化输出**: 每个错误变体都有自定义的错误消息格式
3. **调试支持**: 实现了 `Debug` trait，便于调试
4. **thiserror集成**: 使用 `thiserror` crate 简化错误定义

## 使用示例

```rust
use crate::error::Aria2Error;

// 手动创建RPC错误
let rpc_error = Aria2Error::RpcError(1, "Method not found".to_string());

// 自动转换IO错误
let io_result: Result<(), std::io::Error> = Err(std::io::Error::new(
    std::io::ErrorKind::NotFound,
    "File not found"
));
let aria2_result: Result<(), Aria2Error> = io_result.map_err(Into::into);

// 模式匹配处理错误
match some_result {
    Err(Aria2Error::DaemonUnavailable(msg)) => {
        println!("Daemon error: {}", msg);
    }
    Err(Aria2Error::TaskNotFound(task_id)) => {
        println!("Task {} not found", task_id);
    }
    _ => {}
}
```