//! # BurnCloud Aria2 下载库
//!
//! 这是一个简单的 Rust 库，用于下载、配置和管理 aria2 下载器。
//! 遵循"极度简单"的设计原则，所有功能都在此文件中实现。

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// 常量定义
const DEFAULT_PORT: u16 = 6800;
const MAX_PORT_RANGE: u16 = 100;
const ARIA2_MAIN_URL: &str = "https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip";
const ARIA2_BACKUP_URL: &str = "https://gitee.com/burncloud/aria2/raw/master/aria2-1.37.0-win-64bit-build1.zip";

// ============================================================================
// 错误类型定义
// ============================================================================

#[derive(Debug)]
pub enum Aria2Error {
    DownloadError(String),
    PortError(String),
    RpcError(String),
    DaemonError(String),
    ProcessError(String),
    ConfigError(String),
}

impl std::fmt::Display for Aria2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aria2Error::DownloadError(msg) => write!(f, "下载错误: {}", msg),
            Aria2Error::PortError(msg) => write!(f, "端口错误: {}", msg),
            Aria2Error::RpcError(msg) => write!(f, "RPC错误: {}", msg),
            Aria2Error::DaemonError(msg) => write!(f, "守护进程错误: {}", msg),
            Aria2Error::ProcessError(msg) => write!(f, "进程错误: {}", msg),
            Aria2Error::ConfigError(msg) => write!(f, "配置错误: {}", msg),
        }
    }
}

impl std::error::Error for Aria2Error {}

pub type Aria2Result<T> = Result<T, Aria2Error>;

// ============================================================================
// 数据结构定义
// ============================================================================

#[derive(Debug, Clone)]
pub struct Aria2Config {
    pub port: u16,
    pub secret: Option<String>,
    pub download_dir: PathBuf,
    pub max_connections: u8,
    pub split_size: String,
    pub aria2_path: PathBuf,
}

impl Default for Aria2Config {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            secret: None,
            download_dir: std::env::current_dir().unwrap_or_default().join("downloads"),
            max_connections: 16,
            split_size: "1M".to_string(),
            aria2_path: PathBuf::from(r"C:\Users\username\AppData\Local\BurnCloud\aria2c.exe"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<u8>,
    #[serde(rename = "max-connection-per-server", skip_serializing_if = "Option::is_none")]
    pub max_connection_per_server: Option<u8>,
    #[serde(rename = "continue", skip_serializing_if = "Option::is_none")]
    pub continue_download: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DownloadStatus {
    pub gid: String,
    pub status: String,
    #[serde(rename = "totalLength")]
    pub total_length: String,
    #[serde(rename = "completedLength")]
    pub completed_length: String,
    #[serde(rename = "downloadSpeed")]
    pub download_speed: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalStat {
    #[serde(rename = "downloadSpeed")]
    pub download_speed: String,
    #[serde(rename = "numActive")]
    pub num_active: String,
    #[serde(rename = "numWaiting")]
    pub num_waiting: String,
}

pub struct Aria2Instance {
    pub process: Child,
    pub port: u16,
    pub config: Aria2Config,
}

impl Aria2Instance {
    pub fn is_running(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }

    pub fn kill(&mut self) -> Aria2Result<()> {
        self.process.kill()
            .map_err(|e| Aria2Error::ProcessError(e.to_string()))?;
        self.process.wait()
            .map_err(|e| Aria2Error::ProcessError(e.to_string()))?;
        Ok(())
    }
}

// ============================================================================
// Aria2 下载功能
// ============================================================================

/// 下载 aria2 二进制文件
pub async fn download_aria2() -> Aria2Result<PathBuf> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| Aria2Error::DownloadError(e.to_string()))?;

    let target_dir = PathBuf::from(r"C:\Users\username\AppData\Local\BurnCloud");
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| Aria2Error::DownloadError(format!("创建目录失败: {}", e)))?;

    let zip_path = target_dir.join("aria2.zip");
    let exe_path = target_dir.join("aria2c.exe");

    // 如果 exe 已存在，直接返回
    if exe_path.exists() {
        return Ok(exe_path);
    }

    // 尝试主链接下载
    match download_file(&client, ARIA2_MAIN_URL, &zip_path).await {
        Ok(_) => println!("从主链接下载成功"),
        Err(_) => {
            println!("主链接下载失败，尝试备用链接...");
            download_file(&client, ARIA2_BACKUP_URL, &zip_path).await
                .map_err(|e| Aria2Error::DownloadError(format!("所有下载链接均失败: {}", e)))?;
            println!("从备用链接下载成功");
        }
    }

    // 解压 ZIP 文件
    extract_aria2(&zip_path, &target_dir)?;

    // 删除 ZIP 文件
    let _ = std::fs::remove_file(&zip_path);

    if exe_path.exists() {
        Ok(exe_path)
    } else {
        Err(Aria2Error::DownloadError("解压后未找到 aria2c.exe".to_string()))
    }
}

async fn download_file(client: &Client, url: &str, path: &Path) -> Aria2Result<()> {
    let response = client.get(url).send().await
        .map_err(|e| Aria2Error::DownloadError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(Aria2Error::DownloadError(format!("HTTP错误: {}", response.status())));
    }

    let bytes = response.bytes().await
        .map_err(|e| Aria2Error::DownloadError(e.to_string()))?;

    std::fs::write(path, &bytes)
        .map_err(|e| Aria2Error::DownloadError(e.to_string()))?;

    Ok(())
}

fn extract_aria2(zip_path: &Path, target_dir: &Path) -> Aria2Result<()> {
    let file = std::fs::File::open(zip_path)
        .map_err(|e| Aria2Error::DownloadError(e.to_string()))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| Aria2Error::DownloadError(e.to_string()))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| Aria2Error::DownloadError(e.to_string()))?;

        if file.name() == "aria2c.exe" {
            let mut out_file = std::fs::File::create(target_dir.join("aria2c.exe"))
                .map_err(|e| Aria2Error::DownloadError(e.to_string()))?;
            std::io::copy(&mut file, &mut out_file)
                .map_err(|e| Aria2Error::DownloadError(e.to_string()))?;
            return Ok(());
        }
    }

    Err(Aria2Error::DownloadError("ZIP文件中未找到 aria2c.exe".to_string()))
}

// ============================================================================
// 端口管理
// ============================================================================

/// 检查端口是否可用
pub fn check_port_available(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// 查找可用端口
pub fn find_available_port() -> Aria2Result<u16> {
    for port in DEFAULT_PORT..=(DEFAULT_PORT + MAX_PORT_RANGE) {
        if check_port_available(port) {
            return Ok(port);
        }
    }
    Err(Aria2Error::PortError("未找到可用端口".to_string()))
}

/// 启动 aria2 RPC 服务
pub async fn start_aria2_rpc(config: &Aria2Config) -> Aria2Result<Aria2Instance> {
    let port = find_available_port()?;

    let mut cmd = Command::new(&config.aria2_path);
    cmd.args([
        "--enable-rpc",
        "--rpc-listen-all",
        &format!("--rpc-listen-port={}", port),
        &format!("--dir={}", config.download_dir.display()),
        &format!("--max-connection-per-server={}", config.max_connections),
        &format!("--split={}", config.max_connections),
        &format!("--min-split-size={}", config.split_size),
        "--continue=true",
        "--max-tries=0",
        "--retry-wait=3",
        "--daemon=true",
    ]);

    if let Some(secret) = &config.secret {
        cmd.arg(&format!("--rpc-secret={}", secret));
    }

    let child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Aria2Error::ProcessError(e.to_string()))?;

    let instance = Aria2Instance {
        process: child,
        port,
        config: config.clone(),
    };

    // 等待 RPC 服务启动
    wait_for_rpc_ready(port, &config.secret).await?;

    Ok(instance)
}

async fn wait_for_rpc_ready(port: u16, secret: &Option<String>) -> Aria2Result<()> {
    let client = Client::new();
    let url = format!("http://localhost:{}/jsonrpc", port);

    for _ in 0..30 {
        let mut params = vec![];
        if let Some(s) = secret {
            params.push(Value::String(format!("token:{}", s)));
        }

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "test",
            "method": "aria2.getVersion",
            "params": params
        });

        if let Ok(response) = client.post(&url).json(&request).send().await {
            if response.status().is_success() {
                return Ok(());
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Err(Aria2Error::RpcError("RPC 服务启动超时".to_string()))
}

// ============================================================================
// RPC 客户端
// ============================================================================

pub struct Aria2RpcClient {
    client: Client,
    base_url: String,
    secret: Option<String>,
    request_id: Arc<AtomicU64>,
}

impl Aria2RpcClient {
    pub fn new(port: u16, secret: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: format!("http://localhost:{}/jsonrpc", port),
            secret,
            request_id: Arc::new(AtomicU64::new(1)),
        }
    }

    async fn call_method<T, R>(&self, method: &str, params: T) -> Aria2Result<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let mut rpc_params = Vec::new();

        // 添加 secret（如果配置了）
        if let Some(secret) = &self.secret {
            rpc_params.push(Value::String(format!("token:{}", secret)));
        }

        // 添加其他参数
        let param_value = serde_json::to_value(&params)
            .map_err(|e| Aria2Error::RpcError(e.to_string()))?;
        if !param_value.is_null() {
            rpc_params.push(param_value);
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
            .await
            .map_err(|e| Aria2Error::RpcError(e.to_string()))?;

        let rpc_response: Value = response.json().await
            .map_err(|e| Aria2Error::RpcError(e.to_string()))?;

        if let Some(error) = rpc_response.get("error") {
            return Err(Aria2Error::RpcError(format!("服务器错误: {}", error)));
        }

        let result = rpc_response["result"].clone();
        serde_json::from_value(result)
            .map_err(|e| Aria2Error::RpcError(e.to_string()))
    }

    /// 添加 URI 下载任务
    pub async fn add_uri(&self, uris: Vec<String>, options: Option<DownloadOptions>) -> Aria2Result<String> {
        if let Some(opts) = options {
            self.call_method("aria2.addUri", (uris, serde_json::json!({}), opts)).await
        } else {
            self.call_method("aria2.addUri", vec![uris]).await
        }
    }

    /// 获取下载状态
    pub async fn tell_status(&self, gid: &str) -> Aria2Result<DownloadStatus> {
        self.call_method("aria2.tellStatus", vec![gid]).await
    }

    /// 获取活跃下载列表
    pub async fn tell_active(&self) -> Aria2Result<Vec<DownloadStatus>> {
        self.call_method("aria2.tellActive", ()).await
    }

    /// 获取全局统计信息
    pub async fn get_global_stat(&self) -> Aria2Result<GlobalStat> {
        self.call_method("aria2.getGlobalStat", ()).await
    }

    /// 暂停下载
    pub async fn pause(&self, gid: &str) -> Aria2Result<String> {
        self.call_method("aria2.pause", vec![gid]).await
    }

    /// 恢复下载
    pub async fn unpause(&self, gid: &str) -> Aria2Result<String> {
        self.call_method("aria2.unpause", vec![gid]).await
    }

    /// 移除下载
    pub async fn remove(&self, gid: &str) -> Aria2Result<String> {
        self.call_method("aria2.remove", vec![gid]).await
    }

    /// 关闭 aria2
    pub async fn shutdown(&self) -> Aria2Result<String> {
        self.call_method("aria2.shutdown", ()).await
    }
}

// ============================================================================
// 简单守护进程
// ============================================================================

pub struct Aria2Daemon {
    instance: Option<Aria2Instance>,
    config: Aria2Config,
    is_running: Arc<AtomicBool>,
}

impl Aria2Daemon {
    pub fn new(config: Aria2Config) -> Self {
        Self {
            instance: None,
            config,
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(&mut self) -> Aria2Result<()> {
        if self.is_running.load(Ordering::SeqCst) {
            return Err(Aria2Error::DaemonError("守护进程已在运行".to_string()));
        }

        let instance = start_aria2_rpc(&self.config).await?;
        println!("aria2 RPC 服务已启动在端口: {}", instance.port);

        self.instance = Some(instance);
        self.is_running.store(true, Ordering::SeqCst);

        Ok(())
    }

    pub async fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);

        if let Some(ref mut instance) = self.instance {
            let _ = instance.kill();
        }

        self.instance = None;
        println!("aria2 守护进程已停止");
    }

    pub fn get_rpc_client(&self) -> Option<Aria2RpcClient> {
        self.instance.as_ref().map(|instance| {
            Aria2RpcClient::new(instance.port, self.config.secret.clone())
        })
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

// ============================================================================
// 统一管理器 - 主要入口点
// ============================================================================

pub struct Aria2Manager {
    daemon: Option<Aria2Daemon>,
    config: Aria2Config,
}

impl Aria2Manager {
    pub fn new() -> Self {
        Self {
            daemon: None,
            config: Aria2Config::default(),
        }
    }

    pub fn with_config(config: Aria2Config) -> Self {
        Self {
            daemon: None,
            config,
        }
    }

    /// 下载并设置 aria2
    pub async fn download_and_setup(&mut self) -> Aria2Result<()> {
        println!("正在下载 aria2...");
        let aria2_path = download_aria2().await?;
        println!("aria2 已下载到: {:?}", aria2_path);

        self.config.aria2_path = aria2_path;
        Ok(())
    }

    /// 启动守护进程
    pub async fn start_daemon(&mut self) -> Aria2Result<()> {
        if self.daemon.is_some() {
            return Err(Aria2Error::DaemonError("守护进程已存在".to_string()));
        }

        let mut daemon = Aria2Daemon::new(self.config.clone());
        daemon.start().await?;
        self.daemon = Some(daemon);

        println!("aria2 守护进程启动成功！");
        Ok(())
    }

    /// 获取 RPC 客户端
    pub fn get_rpc_client(&self) -> Option<&Aria2RpcClient> {
        // 由于借用检查器限制，这里简化实现
        None
    }

    /// 创建新的 RPC 客户端
    pub fn create_rpc_client(&self) -> Option<Aria2RpcClient> {
        self.daemon.as_ref().and_then(|d| d.get_rpc_client())
    }

    /// 关闭管理器
    pub async fn shutdown(&mut self) -> Aria2Result<()> {
        if let Some(ref mut daemon) = self.daemon {
            daemon.stop().await;
        }
        self.daemon = None;
        println!("Aria2Manager 已关闭");
        Ok(())
    }

    /// 检查是否运行中
    pub fn is_running(&self) -> bool {
        self.daemon.as_ref().map_or(false, |d| d.is_running())
    }
}

impl Default for Aria2Manager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 便利函数
// ============================================================================

/// 快速启动 aria2 管理器
pub async fn quick_start() -> Aria2Result<Aria2Manager> {
    let mut manager = Aria2Manager::new();
    manager.download_and_setup().await?;
    manager.start_daemon().await?;
    Ok(manager)
}