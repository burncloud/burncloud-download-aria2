# 管理器模块 (manager/)

## 概述

管理器模块是 BurnCloud Aria2 Download Manager 的核心组件，实现了 `DownloadManager` trait，为上层应用提供统一的下载管理接口。该模块负责任务管理、进度追踪、状态同步等核心功能。

## 核心组件

### `Aria2DownloadManager`

主要的下载管理器实现，集成了客户端、轮询器和守护进程管理。

#### 结构定义
```rust
pub struct Aria2DownloadManager {
    client: Arc<Aria2Client>,                    // JSON-RPC 客户端
    _poller: Arc<ProgressPoller>,               // 进度轮询器
    _daemon: Arc<crate::daemon::Aria2Daemon>,   // 守护进程管理器
    task_gid_map: Arc<tokio::sync::RwLock<HashMap<TaskId, String>>>, // TaskId 到 GID 的映射
}
```

**字段说明**:
- `client`: 与 Aria2 通信的 JSON-RPC 客户端
- `_poller`: 后台进度轮询器（用于扩展功能）
- `_daemon`: Aria2 守护进程管理器
- `task_gid_map`: TaskId 与 Aria2 GID 的双向映射关系

## 核心功能

### 1. 管理器创建

#### `new(rpc_url: String, secret: Option<String>) -> Result<Self>`

**功能**: 创建新的下载管理器实例

**实现流程**:
1. **客户端创建**: 创建 JSON-RPC 客户端
2. **端口解析**: 从 RPC URL 中提取端口号
3. **守护进程配置**: 创建守护进程配置
4. **端口冲突检测**: 确保端口可用，如有冲突自动寻找下一个可用端口
5. **客户端更新**: 如果端口发生变化，更新客户端配置
6. **守护进程启动**: 启动 Aria2 守护进程
7. **轮询器初始化**: 启动进度轮询器

**自动端口解决机制**:
```rust
// 端口冲突检测和解决
daemon_config = daemon_config.ensure_available_port().await?;

// 如果端口改变，更新客户端
if daemon_config.rpc_port != rpc_port {
    let new_url = Self::update_url_port(&rpc_url, daemon_config.rpc_port);
    println!("Updated RPC URL to: {}", new_url);
    final_client = Arc::new(Aria2Client::new(new_url, daemon_config.rpc_secret.clone().into()));
}
```

### 2. URL 处理工具

#### `extract_port_from_url(url: &str) -> Option<u16>`
- **功能**: 从 RPC URL 中提取端口号
- **示例**: `"http://localhost:6800/jsonrpc"` → `Some(6800)`

#### `update_url_port(url: &str, new_port: u16) -> String`
- **功能**: 更新 URL 中的端口号
- **示例**: `"http://localhost:6800/jsonrpc"` + `6801` → `"http://localhost:6801/jsonrpc"`

### 3. 下载类型检测

#### `detect_download_type(&self, url: &str) -> Result<DownloadType>`

**功能**: 根据 URL 自动检测下载类型

**支持的类型**:
```rust
enum DownloadType {
    Http,       // HTTP/HTTPS/FTP 链接
    Torrent,    // .torrent 文件
    Metalink,   // .metalink/.meta4 文件
    Magnet,     // magnet: 链接
}
```

**检测逻辑**:
- `magnet:` 开头 → Magnet 链接
- `.torrent` 结尾 → BitTorrent 文件
- `.metalink`/`.meta4` 结尾 → Metalink 文件
- `http://`/`https://`/`ftp://` 开头 → HTTP 下载

## DownloadManager Trait 实现

### `add_download(url: String, target_path: PathBuf) -> Result<TaskId>`

**功能**: 添加新的下载任务

**实现流程**:
1. **重复检测**: 检查是否已存在相同 URL 的任务
2. **任务创建**: 创建新的 `DownloadTask`
3. **目录创建**: 确保目标目录存在
4. **类型检测**: 自动检测下载类型
5. **配置构建**: 构建 Aria2 下载选项
6. **任务添加**: 根据类型调用相应的 Aria2 方法
7. **映射存储**: 保存 TaskId 到 GID 的映射关系

**重复检测机制**:
```rust
// 遍历所有现有任务，检查 URL 重复
for aria2_task in &existing_tasks {
    if let Some(files) = aria2_task.get("files").and_then(|f| f.as_array()) {
        for file in files {
            if let Some(uris) = file.get("uris").and_then(|u| u.as_array()) {
                for uri_obj in uris {
                    if let Some(existing_url) = uri_obj.get("uri").and_then(|u| u.as_str()) {
                        if existing_url == url {
                            return Ok(existing_task_id); // 返回已存在的任务ID
                        }
                    }
                }
            }
        }
    }
}
```

**根据类型添加任务**:
```rust
let gid = match download_type {
    DownloadType::Http | DownloadType::Magnet => {
        self.client.add_uri(vec![url.clone()], options).await?
    }
    DownloadType::Torrent => {
        let torrent_data = reqwest::get(&url).await?.bytes().await?.to_vec();
        self.client.add_torrent(torrent_data, options).await?
    }
    DownloadType::Metalink => {
        let metalink_data = reqwest::get(&url).await?.bytes().await?.to_vec();
        self.client.add_metalink(metalink_data, options).await?
    }
};
```

### `pause_download(task_id: TaskId) -> Result<()>`

**功能**: 暂停指定的下载任务

**实现**:
1. 从映射表中查找对应的 GID
2. 调用客户端的 `pause()` 方法

### `resume_download(task_id: TaskId) -> Result<()>`

**功能**: 恢复暂停的下载任务

**实现**:
1. 从映射表中查找对应的 GID
2. 调用客户端的 `unpause()` 方法

### `cancel_download(task_id: TaskId) -> Result<()>`

**功能**: 取消并删除下载任务

**实现**:
1. 从映射表中移除 TaskId 和 GID 的映射
2. 调用客户端的 `remove()` 方法删除任务

### `get_progress(task_id: TaskId) -> Result<DownloadProgress>`

**功能**: 获取任务的实时下载进度

**实现**:
1. 查找对应的 GID
2. 调用 `client.tell_status()` 获取实时状态
3. 解析 JSON 数据提取进度信息
4. 计算 ETA（预计完成时间）

**进度信息提取**:
```rust
let total_bytes = status
    .get("totalLength")
    .and_then(|v| v.as_str())
    .and_then(|s| s.parse::<u64>().ok())
    .unwrap_or(0);

let downloaded_bytes = status
    .get("completedLength")
    .and_then(|v| v.as_str())
    .and_then(|s| s.parse::<u64>().ok())
    .unwrap_or(0);

let speed_bps = status
    .get("downloadSpeed")
    .and_then(|v| v.as_str())
    .and_then(|s| s.parse::<u64>().ok())
    .unwrap_or(0);

// 计算 ETA
let eta_seconds = if speed_bps > 0 && total_bytes > downloaded_bytes {
    Some((total_bytes - downloaded_bytes) / speed_bps)
} else {
    None
};
```

### `get_task(task_id: TaskId) -> Result<DownloadTask>`

**功能**: 获取任务的详细信息

**实现**:
1. 查找对应的 GID
2. 获取 Aria2 状态信息
3. 从状态中重构 `DownloadTask` 对象

### `list_tasks() -> Result<Vec<DownloadTask>>`

**功能**: 获取所有任务列表

**实现流程**:
1. **获取所有 Aria2 任务**: 包括活跃、等待和已停止的任务
2. **映射匹配**: 将 Aria2 的 GID 与内部 TaskId 进行匹配
3. **任务重构**: 从 Aria2 状态重构 `DownloadTask` 对象

**获取所有任务**:
```rust
async fn get_all_aria2_tasks(&self) -> Result<Vec<serde_json::Value>> {
    let mut all_tasks = Vec::new();

    // 获取活跃下载
    if let Ok(active) = self.client.tell_active().await {
        if let Some(active_array) = active.as_array() {
            all_tasks.extend(active_array.clone());
        }
    }

    // 获取等待中的下载（限制1000个）
    if let Ok(waiting) = self.client.tell_waiting(0, 1000).await {
        if let Some(waiting_array) = waiting.as_array() {
            all_tasks.extend(waiting_array.clone());
        }
    }

    // 获取已停止的下载（限制1000个）
    if let Ok(stopped) = self.client.tell_stopped(0, 1000).await {
        if let Some(stopped_array) = stopped.as_array() {
            all_tasks.extend(stopped_array.clone());
        }
    }

    Ok(all_tasks)
}
```

### `active_download_count() -> Result<usize>`

**功能**: 获取当前活跃下载任务数量

**实现**:
1. 调用 `client.tell_active()` 获取活跃任务
2. 统计 JSON 数组中的元素数量

## 设计特点

### 实时数据访问

- **直接 JSON 解析**: 避免中间结构体转换，保证数据实时性
- **按需查询**: 每次都向 Aria2 查询最新状态
- **无缓存设计**: 确保数据的准确性和一致性

### 任务映射管理

- **双向映射**: TaskId ↔ GID 的映射关系
- **并发安全**: 使用 `RwLock` 保护映射表
- **生命周期管理**: 任务删除时自动清理映射

### 错误处理

- **优雅降级**: 部分操作失败不影响整体功能
- **详细错误信息**: 提供调试友好的错误消息
- **自动重试**: 网络错误时的自动重试机制

### 异步设计

- **非阻塞操作**: 所有 I/O 操作都是异步的
- **并发支持**: 支持多任务并发管理
- **资源共享**: 使用 `Arc` 实现安全的资源共享

## 测试用例

模块包含完整的单元测试：

### URL 处理测试
```rust
#[test]
fn test_extract_port_from_url() {
    assert_eq!(Aria2DownloadManager::extract_port_from_url("http://localhost:6800/jsonrpc"), Some(6800));
    assert_eq!(Aria2DownloadManager::extract_port_from_url("http://localhost:9999/jsonrpc"), Some(9999));
    assert_eq!(Aria2DownloadManager::extract_port_from_url("http://localhost/jsonrpc"), None);
}

#[test]
fn test_update_url_port() {
    assert_eq!(
        Aria2DownloadManager::update_url_port("http://localhost:6800/jsonrpc", 6801),
        "http://localhost:6801/jsonrpc"
    );
}
```

## 使用示例

### 基本使用流程

```rust
use burncloud_download_aria2::Aria2DownloadManager;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建管理器
    let manager = Aria2DownloadManager::new(
        "http://localhost:6800/jsonrpc".to_string(),
        Some("burncloud".to_string())
    ).await?;

    // 添加下载任务
    let task_id = manager.add_download(
        "https://example.com/file.zip".to_string(),
        PathBuf::from("/downloads/file.zip")
    ).await?;

    println!("任务已添加: {:?}", task_id);

    // 监控进度
    loop {
        let progress = manager.get_progress(task_id).await?;
        println!("进度: {}/{} bytes, 速度: {} B/s",
            progress.downloaded_bytes,
            progress.total_bytes.unwrap_or(0),
            progress.speed_bps
        );

        if progress.downloaded_bytes == progress.total_bytes.unwrap_or(0) && progress.total_bytes.is_some() {
            println!("下载完成！");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
```

## 相关文档

- [客户端模块](../client/) - JSON-RPC 客户端实现
- [守护进程模块](../daemon/) - Aria2 守护进程管理
- [轮询器模块](../poller/) - 进度轮询机制
- [错误处理](../error.md) - 错误类型和处理策略