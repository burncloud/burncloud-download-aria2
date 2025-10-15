# Aria2 RPC 接口功能文档

## 概述

本模块实现完整的 aria2 JSON-RPC 2.0 接口，提供所有下载管理功能。

## RPC 客户端架构

### 客户端结构
```rust
pub struct Aria2RpcClient {
    client: reqwest::Client,
    base_url: String,
    secret: Option<String>,
    request_id: Arc<AtomicU64>,
}

impl Aria2RpcClient {
    pub fn new(port: u16, secret: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: format!("http://localhost:{}/jsonrpc", port),
            secret,
            request_id: Arc::new(AtomicU64::new(1)),
        }
    }
}
```

### 通用 RPC 调用方法
```rust
impl Aria2RpcClient {
    async fn call_method<T, R>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, RpcError>
    where
        T: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let mut rpc_params = Vec::new();

        // 添加 secret（如果配置了）
        if let Some(secret) = &self.secret {
            rpc_params.push(serde_json::Value::String(format!("token:{}", secret)));
        }

        // 添加其他参数
        if !serde_json::to_value(&params)?.is_null() {
            rpc_params.push(serde_json::to_value(&params)?);
        }

        let request_id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id.to_string(),
            "method": method,
            "params": rpc_params
        });

        let response = self.client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await?;

        let rpc_response: serde_json::Value = response.json().await?;

        if let Some(error) = rpc_response.get("error") {
            return Err(RpcError::ServerError(error.clone()));
        }

        let result = rpc_response["result"].clone();
        Ok(serde_json::from_value(result)?)
    }
}
```

## 核心 RPC 接口实现

### 1. 下载任务管理

#### 添加下载任务
```rust
impl Aria2RpcClient {
    /// 添加 URI 下载任务
    pub async fn add_uri(
        &self,
        uris: Vec<String>,
        options: Option<DownloadOptions>,
    ) -> Result<String, RpcError> {
        let params = if let Some(opts) = options {
            vec![uris, vec![serde_json::to_value(opts)?]]
        } else {
            vec![uris]
        };

        self.call_method("aria2.addUri", params).await
    }

    /// 添加种子文件下载
    pub async fn add_torrent(
        &self,
        torrent_data: Vec<u8>,
        options: Option<DownloadOptions>,
    ) -> Result<String, RpcError> {
        let encoded = base64::encode(torrent_data);
        let params = if let Some(opts) = options {
            vec![encoded, vec![], serde_json::to_value(opts)?]
        } else {
            vec![encoded]
        };

        self.call_method("aria2.addTorrent", params).await
    }

    /// 添加磁力链接下载
    pub async fn add_metalink(
        &self,
        metalink_data: Vec<u8>,
        options: Option<DownloadOptions>,
    ) -> Result<Vec<String>, RpcError> {
        let encoded = base64::encode(metalink_data);
        let params = if let Some(opts) = options {
            vec![encoded, serde_json::to_value(opts)?]
        } else {
            vec![encoded]
        };

        self.call_method("aria2.addMetalink", params).await
    }
}
```

#### 下载任务控制
```rust
impl Aria2RpcClient {
    /// 暂停下载
    pub async fn pause(&self, gid: &str) -> Result<String, RpcError> {
        self.call_method("aria2.pause", vec![gid]).await
    }

    /// 暂停所有下载
    pub async fn pause_all(&self) -> Result<String, RpcError> {
        self.call_method("aria2.pauseAll", ()).await
    }

    /// 强制暂停下载
    pub async fn force_pause(&self, gid: &str) -> Result<String, RpcError> {
        self.call_method("aria2.forcePause", vec![gid]).await
    }

    /// 强制暂停所有下载
    pub async fn force_pause_all(&self) -> Result<String, RpcError> {
        self.call_method("aria2.forcePauseAll", ()).await
    }

    /// 恢复下载
    pub async fn unpause(&self, gid: &str) -> Result<String, RpcError> {
        self.call_method("aria2.unpause", vec![gid]).await
    }

    /// 恢复所有下载
    pub async fn unpause_all(&self) -> Result<String, RpcError> {
        self.call_method("aria2.unpauseAll", ()).await
    }

    /// 移除下载
    pub async fn remove(&self, gid: &str) -> Result<String, RpcError> {
        self.call_method("aria2.remove", vec![gid]).await
    }

    /// 强制移除下载
    pub async fn force_remove(&self, gid: &str) -> Result<String, RpcError> {
        self.call_method("aria2.forceRemove", vec![gid]).await
    }

    /// 从结果中移除下载记录
    pub async fn remove_download_result(&self, gid: &str) -> Result<String, RpcError> {
        self.call_method("aria2.removeDownloadResult", vec![gid]).await
    }
}
```

### 2. 状态查询接口

#### 获取下载状态
```rust
impl Aria2RpcClient {
    /// 获取下载状态
    pub async fn tell_status(
        &self,
        gid: &str,
        keys: Option<Vec<String>>,
    ) -> Result<DownloadStatus, RpcError> {
        let params = if let Some(k) = keys {
            vec![gid.to_string(), k]
        } else {
            vec![gid.to_string()]
        };

        self.call_method("aria2.tellStatus", params).await
    }

    /// 获取活跃下载列表
    pub async fn tell_active(
        &self,
        keys: Option<Vec<String>>,
    ) -> Result<Vec<DownloadStatus>, RpcError> {
        let params = keys.unwrap_or_default();
        self.call_method("aria2.tellActive", params).await
    }

    /// 获取等待下载列表
    pub async fn tell_waiting(
        &self,
        offset: i32,
        num: i32,
        keys: Option<Vec<String>>,
    ) -> Result<Vec<DownloadStatus>, RpcError> {
        let params = if let Some(k) = keys {
            vec![offset, num, k]
        } else {
            vec![offset, num]
        };

        self.call_method("aria2.tellWaiting", params).await
    }

    /// 获取已停止下载列表
    pub async fn tell_stopped(
        &self,
        offset: i32,
        num: i32,
        keys: Option<Vec<String>>,
    ) -> Result<Vec<DownloadStatus>, RpcError> {
        let params = if let Some(k) = keys {
            vec![offset, num, k]
        } else {
            vec![offset, num]
        };

        self.call_method("aria2.tellStopped", params).await
    }
}
```

### 3. 全局统计和配置

#### 统计信息
```rust
impl Aria2RpcClient {
    /// 获取全局统计信息
    pub async fn get_global_stat(&self) -> Result<GlobalStat, RpcError> {
        self.call_method("aria2.getGlobalStat", ()).await
    }

    /// 清除已完成/错误/移除的下载结果
    pub async fn purge_download_result(&self) -> Result<String, RpcError> {
        self.call_method("aria2.purgeDownloadResult", ()).await
    }
}
```

#### 配置管理
```rust
impl Aria2RpcClient {
    /// 获取全局选项
    pub async fn get_global_option(&self) -> Result<GlobalOption, RpcError> {
        self.call_method("aria2.getGlobalOption", ()).await
    }

    /// 修改全局选项
    pub async fn change_global_option(
        &self,
        options: GlobalOption,
    ) -> Result<String, RpcError> {
        self.call_method("aria2.changeGlobalOption", vec![options]).await
    }

    /// 获取下载选项
    pub async fn get_option(&self, gid: &str) -> Result<DownloadOptions, RpcError> {
        self.call_method("aria2.getOption", vec![gid]).await
    }

    /// 修改下载选项
    pub async fn change_option(
        &self,
        gid: &str,
        options: DownloadOptions,
    ) -> Result<String, RpcError> {
        self.call_method("aria2.changeOption", vec![gid, options]).await
    }
}
```

### 4. 系统信息接口

#### 版本和会话信息
```rust
impl Aria2RpcClient {
    /// 获取版本信息
    pub async fn get_version(&self) -> Result<VersionInfo, RpcError> {
        self.call_method("aria2.getVersion", ()).await
    }

    /// 获取会话信息
    pub async fn get_session_info(&self) -> Result<SessionInfo, RpcError> {
        self.call_method("aria2.getSessionInfo", ()).await
    }

    /// 关闭 aria2
    pub async fn shutdown(&self) -> Result<String, RpcError> {
        self.call_method("aria2.shutdown", ()).await
    }

    /// 强制关闭 aria2
    pub async fn force_shutdown(&self) -> Result<String, RpcError> {
        self.call_method("aria2.forceShutdown", ()).await
    }
}
```

## 数据结构定义

### 下载选项
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadOptions {
    pub dir: Option<String>,                      // 下载目录
    pub out: Option<String>,                      // 输出文件名
    pub split: Option<u8>,                        // 分片数
    #[serde(rename = "max-connection-per-server")]
    pub max_connection_per_server: Option<u8>,    // 每服务器最大连接数
    #[serde(rename = "min-split-size")]
    pub min_split_size: Option<String>,           // 最小分片大小
    #[serde(rename = "max-download-limit")]
    pub max_download_limit: Option<String>,       // 最大下载速度
    #[serde(rename = "max-upload-limit")]
    pub max_upload_limit: Option<String>,         // 最大上传速度
    pub continue_: Option<bool>,                  // 断点续传
    // 更多选项...
}
```

### 下载状态
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct DownloadStatus {
    pub gid: String,                              // 全局ID
    pub status: String,                           // 状态：active/waiting/paused/error/complete/removed
    #[serde(rename = "totalLength")]
    pub total_length: String,                     // 总大小
    #[serde(rename = "completedLength")]
    pub completed_length: String,                 // 已完成大小
    #[serde(rename = "downloadSpeed")]
    pub download_speed: String,                   // 下载速度
    #[serde(rename = "uploadSpeed")]
    pub upload_speed: String,                     // 上传速度
    pub dir: String,                              // 下载目录
    pub files: Vec<FileInfo>,                     // 文件信息
    // 更多字段...
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub index: String,                            // 文件索引
    pub path: String,                             // 文件路径
    pub length: String,                           // 文件大小
    #[serde(rename = "completedLength")]
    pub completed_length: String,                 // 已完成大小
    pub selected: String,                         // 是否选中
    pub uris: Vec<UriInfo>,                       // URI 信息
}
```

### 全局统计
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct GlobalStat {
    #[serde(rename = "downloadSpeed")]
    pub download_speed: String,                   // 总下载速度
    #[serde(rename = "uploadSpeed")]
    pub upload_speed: String,                     // 总上传速度
    #[serde(rename = "numActive")]
    pub num_active: String,                       // 活跃下载数
    #[serde(rename = "numWaiting")]
    pub num_waiting: String,                      // 等待下载数
    #[serde(rename = "numStopped")]
    pub num_stopped: String,                      // 已停止下载数
    #[serde(rename = "numStoppedTotal")]
    pub num_stopped_total: String,                // 总停止下载数
}
```

## 错误处理

### RPC 错误类型
```rust
#[derive(Debug)]
pub enum RpcError {
    NetworkError(reqwest::Error),
    SerializationError(serde_json::Error),
    ServerError(serde_json::Value),
    InvalidResponse(String),
}

impl From<reqwest::Error> for RpcError {
    fn from(err: reqwest::Error) -> Self {
        RpcError::NetworkError(err)
    }
}

impl From<serde_json::Error> for RpcError {
    fn from(err: serde_json::Error) -> Self {
        RpcError::SerializationError(err)
    }
}
```

## 使用示例

### 基础使用
```rust
use crate::Aria2RpcClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Aria2RpcClient::new(6800, None);

    // 添加下载任务
    let gid = client.add_uri(
        vec!["http://example.com/file.zip".to_string()],
        None,
    ).await?;

    println!("下载任务已添加，GID: {}", gid);

    // 获取下载状态
    let status = client.tell_status(&gid, None).await?;
    println!("下载状态: {:?}", status);

    // 获取全局统计
    let global_stat = client.get_global_stat().await?;
    println!("全局统计: {:?}", global_stat);

    Ok(())
}
```

### 高级使用
```rust
// 带选项的下载
let options = DownloadOptions {
    dir: Some("/path/to/downloads".to_string()),
    split: Some(8),
    max_connection_per_server: Some(4),
    ..Default::default()
};

let gid = client.add_uri(
    vec!["http://example.com/largefile.zip".to_string()],
    Some(options),
).await?;

// 监控下载进度
loop {
    let status = client.tell_status(&gid, None).await?;

    match status.status.as_str() {
        "active" => {
            let progress = status.completed_length.parse::<u64>().unwrap_or(0) * 100
                         / status.total_length.parse::<u64>().unwrap_or(1);
            println!("下载进度: {}%", progress);
        }
        "complete" => {
            println!("下载完成！");
            break;
        }
        "error" => {
            println!("下载出错");
            break;
        }
        _ => {}
    }

    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

## 依赖库

### 必需依赖
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
base64 = "0.21"
```

## 注意事项

1. **线程安全**: 客户端支持多线程使用
2. **错误重试**: 网络错误时实现适当的重试机制
3. **资源管理**: 确保正确处理连接和内存资源
4. **安全性**: 生产环境中应配置 RPC secret
5. **性能优化**: 合理设置连接池和超时参数

## 测试要点

1. 所有 RPC 方法的功能测试
2. 错误响应处理
3. 网络异常处理
4. 并发调用测试
5. 大文件下载测试
6. 断点续传测试