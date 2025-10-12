# manager/mod.rs - 下载管理器模块文档

## 概述

`manager/mod.rs` 是下载管理器模块的核心实现，它实现了 `DownloadManager` trait，提供了统一的下载管理接口。该模块将aria2的功能封装为易于使用的高级API。

## 模块结构

### 子模块

- `mapper` - 状态映射模块，处理aria2状态与内部状态的转换

### 主要结构

#### Aria2DownloadManager

主要的下载管理器实现，负责协调所有下载相关的操作。

##### 字段

- `client: Arc<Aria2Client>` - aria2 RPC客户端
- `_poller: Arc<ProgressPoller>` - 进度轮询器（后台运行）
- `_daemon: Arc<crate::daemon::Aria2Daemon>` - aria2守护进程管理器
- `task_gid_map: Arc<tokio::sync::RwLock<HashMap<TaskId, String>>>` - TaskId到GID的映射

##### 特点

- **线程安全**: 使用 `Arc` 和 `RwLock` 确保并发安全
- **自动管理**: 自动启动和管理aria2守护进程
- **状态追踪**: 维护任务ID到aria2 GID的映射关系

#### DownloadType 枚举

下载类型枚举，用于识别不同的下载协议。

```rust
enum DownloadType {
    Http,     // HTTP/HTTPS/FTP下载
    Torrent,  // BitTorrent种子下载
    Metalink, // Metalink文件下载
    Magnet,   // 磁力链接下载
}
```

## 构造和初始化

### new(rpc_url: String, secret: Option<String>) -> Result<Self>

创建新的下载管理器实例。

**参数**:
- `rpc_url`: aria2 RPC服务地址
- `secret`: 可选的RPC认证密钥

**执行流程**:
1. 创建aria2客户端
2. 从RPC URL中提取端口号
3. 配置并启动aria2守护进程
4. 初始化进度轮询器
5. 启动后台进度轮询

**示例**:
```rust
let manager = Aria2DownloadManager::new(
    "http://localhost:6800/jsonrpc".to_string(),
    Some("my_secret".to_string())
).await?;
```

## 私有方法

### extract_port_from_url(url: &str) -> Option<u16>

从RPC URL中提取端口号。

**实现逻辑**:
- 按 `:` 分割URL
- 取第三部分（端口部分）
- 按 `/` 分割取第一部分
- 解析为u16数字

**示例**:
```rust
// "http://localhost:6800/jsonrpc" -> Some(6800)
let port = Self::extract_port_from_url("http://localhost:6800/jsonrpc");
```

### detect_download_type(&self, url: &str) -> Result<DownloadType>

检测URL的下载类型。

**检测规则**:
- `magnet:` 开头 → Magnet
- `.torrent` 结尾 → Torrent
- `.metalink` 或 `.meta4` 结尾 → Metalink
- `http://`, `https://`, `ftp://` 开头 → Http
- 其他 → 错误

### get_all_aria2_tasks(&self) -> Result<Vec<Aria2Status>>

获取所有aria2任务状态。

**实现**:
- 获取活动下载任务
- 获取等待中的任务（限制1000个）
- 获取已停止的任务（限制1000个）
- 合并所有结果

## DownloadManager Trait 实现

### add_download(&self, url: String, target_path: PathBuf) -> Result<TaskId>

添加新的下载任务。

**执行流程**:
1. 创建DownloadTask并生成TaskId
2. 确保目标目录存在
3. 检测下载类型
4. 提取目录和文件名
5. 根据下载类型调用相应的aria2方法
6. 存储TaskId到GID的映射
7. 返回TaskId

**支持的下载类型**:
- **HTTP/Magnet**: 直接调用 `add_uri`
- **Torrent**: 下载种子文件，调用 `add_torrent`
- **Metalink**: 下载metalink文件，调用 `add_metalink`

### pause_download(&self, task_id: TaskId) -> Result<()>

暂停指定的下载任务。

**实现**:
1. 从映射中查找对应的GID
2. 调用aria2客户端的pause方法

### resume_download(&self, task_id: TaskId) -> Result<()>

恢复指定的下载任务。

**实现**:
1. 从映射中查找对应的GID
2. 调用aria2客户端的unpause方法

### cancel_download(&self, task_id: TaskId) -> Result<()>

取消指定的下载任务。

**实现**:
1. 从映射中移除TaskId
2. 调用aria2客户端的remove方法

### get_progress(&self, task_id: TaskId) -> Result<DownloadProgress>

获取下载进度信息。

**计算过程**:
1. 查找对应的GID
2. 获取aria2状态信息
3. 解析字符串格式的数值
4. 计算预估完成时间（ETA）
5. 构造DownloadProgress结构

**ETA计算**:
```rust
let eta_seconds = if speed_bps > 0 && total_bytes > downloaded_bytes {
    Some((total_bytes - downloaded_bytes) / speed_bps)
} else {
    None
};
```

### get_task(&self, task_id: TaskId) -> Result<DownloadTask>

获取指定任务的详细信息。

**重构过程**:
1. 查找对应的GID
2. 获取aria2最新状态
3. 从文件信息中推断目标路径
4. 重新构造DownloadTask
5. 映射状态信息

### list_tasks(&self) -> Result<Vec<DownloadTask>>

列出所有下载任务。

**实现逻辑**:
1. 获取所有aria2任务
2. 遍历任务，查找对应的TaskId
3. 重构每个DownloadTask
4. 返回任务列表

### active_download_count(&self) -> Result<usize>

获取活动下载任务数量。

**实现**: 直接返回aria2活动任务列表的长度

## 设计特点

### 1. 异步架构

- 所有方法都是异步的
- 使用 `tokio` 运行时
- 支持并发操作

### 2. 状态管理

- 维护TaskId到GID的双向映射
- 实时从aria2获取状态信息
- 自动处理状态转换

### 3. 错误处理

- 统一的错误类型转换
- 详细的错误信息
- 优雅的错误传播

### 4. 资源管理

- 自动管理aria2守护进程
- 后台进度轮询
- 线程安全的共享状态

### 5. 协议支持

- HTTP/HTTPS/FTP
- BitTorrent
- Metalink/Meta4
- 磁力链接

## 使用示例

```rust
use crate::manager::Aria2DownloadManager;
use std::path::PathBuf;

// 创建下载管理器
let manager = Aria2DownloadManager::new(
    "http://localhost:6800/jsonrpc".to_string(),
    Some("secret".to_string())
).await?;

// 添加HTTP下载
let task_id = manager.add_download(
    "http://example.com/file.zip".to_string(),
    PathBuf::from("/downloads/file.zip")
).await?;

// 获取进度
let progress = manager.get_progress(task_id).await?;
println!("已下载: {} / {}",
    progress.downloaded_bytes,
    progress.total_bytes.unwrap_or(0)
);

// 暂停下载
manager.pause_download(task_id).await?;

// 恢复下载
manager.resume_download(task_id).await?;

// 列出所有任务
let tasks = manager.list_tasks().await?;
for task in tasks {
    println!("任务 {}: {:?}", task.id, task.status);
}

// 取消下载
manager.cancel_download(task_id).await?;
```

## 依赖关系

- `crate::client::Aria2Client` - aria2 RPC客户端
- `crate::error::Aria2Error` - 错误类型
- `crate::poller::ProgressPoller` - 进度轮询器
- `crate::daemon::Aria2Daemon` - 守护进程管理
- `burncloud_download_types` - 公共类型定义
- `reqwest` - HTTP客户端（用于下载种子/metalink文件）
- `tokio` - 异步运行时
- `async_trait` - 异步trait支持