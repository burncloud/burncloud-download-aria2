# Aria2 守护进程文档

## 概述

本模块实现 aria2 进程的守护功能，确保服务的高可用性和稳定运行。

## 守护进程架构

### 守护进程结构
```rust
pub struct Aria2Daemon {
    instance: Option<Aria2Instance>,
    config: Aria2Config,
    restart_count: u32,
    max_restarts: u32,
    check_interval: Duration,
    is_running: Arc<AtomicBool>,
}

impl Aria2Daemon {
    pub fn new(config: Aria2Config) -> Self {
        Self {
            instance: None,
            config,
            restart_count: 0,
            max_restarts: 10,
            check_interval: Duration::from_secs(5),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }
}
```

### 守护进程配置
```rust
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub check_interval: Duration,        // 健康检查间隔
    pub max_restarts: u32,              // 最大重启次数
    pub restart_delay: Duration,         // 重启延迟
    pub health_check_timeout: Duration,  // 健康检查超时
    pub graceful_shutdown_timeout: Duration, // 优雅关闭超时
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(5),
            max_restarts: 10,
            restart_delay: Duration::from_secs(3),
            health_check_timeout: Duration::from_secs(10),
            graceful_shutdown_timeout: Duration::from_secs(30),
        }
    }
}
```

## 核心守护功能

### 1. 启动守护进程
```rust
impl Aria2Daemon {
    pub async fn start(&mut self, aria2_path: &Path) -> Result<(), DaemonError> {
        if self.is_running.load(Ordering::SeqCst) {
            return Err(DaemonError::AlreadyRunning);
        }

        // 启动 aria2 实例
        let instance = start_aria2_rpc(aria2_path).await?;
        self.instance = Some(instance);
        self.is_running.store(true, Ordering::SeqCst);

        // 启动监控任务
        let daemon_handle = self.start_monitoring().await;

        Ok(())
    }

    async fn start_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let is_running = Arc::clone(&self.is_running);
        let check_interval = self.check_interval;

        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                // 执行健康检查和重启逻辑
                self.health_check_and_restart().await;
                tokio::time::sleep(check_interval).await;
            }
        })
    }
}
```

### 2. 健康检查机制
```rust
impl Aria2Daemon {
    async fn health_check_and_restart(&mut self) {
        if let Some(ref mut instance) = self.instance {
            // 检查进程状态
            if !instance.is_running() {
                println!("检测到 aria2 进程已退出，准备重启...");
                self.handle_process_exit().await;
                return;
            }

            // 检查 RPC 连接
            if !self.check_rpc_health(instance.port).await {
                println!("aria2 RPC 服务无响应，强制重启...");
                self.force_restart().await;
                return;
            }
        } else {
            println!("aria2 实例不存在，尝试重新启动...");
            self.restart_instance().await;
        }
    }

    async fn check_rpc_health(&self, port: u16) -> bool {
        let client = reqwest::Client::builder()
            .timeout(self.config.health_check_timeout)
            .build()
            .unwrap();

        let url = format!("http://localhost:{}/jsonrpc", port);
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "health_check",
            "method": "aria2.getVersion"
        });

        match client.post(&url).json(&request).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}
```

### 3. 自动重启逻辑
```rust
impl Aria2Daemon {
    async fn handle_process_exit(&mut self) {
        if self.restart_count >= self.max_restarts {
            eprintln!("达到最大重启次数 {}，停止守护进程", self.max_restarts);
            self.stop().await;
            return;
        }

        self.restart_count += 1;
        println!("第 {} 次重启 aria2...", self.restart_count);

        // 等待重启延迟
        tokio::time::sleep(self.config.restart_delay).await;

        // 重启实例
        if let Err(e) = self.restart_instance().await {
            eprintln!("重启失败: {:?}", e);
        }
    }

    async fn restart_instance(&mut self) -> Result<(), DaemonError> {
        // 清理旧实例
        if let Some(ref mut instance) = self.instance {
            let _ = instance.kill();
        }
        self.instance = None;

        // 查找可用端口
        let port = find_available_port()?;
        self.config.port = port;

        // 启动新实例
        let new_instance = start_aria2_rpc(&self.config.aria2_path).await?;
        self.instance = Some(new_instance);

        println!("aria2 已重启，端口: {}", port);
        Ok(())
    }

    async fn force_restart(&mut self) {
        if let Some(ref mut instance) = self.instance {
            println!("强制终止 aria2 进程...");
            let _ = instance.kill();
        }

        self.handle_process_exit().await;
    }
}
```

### 4. 优雅关闭
```rust
impl Aria2Daemon {
    pub async fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);

        if let Some(ref mut instance) = self.instance {
            // 尝试优雅关闭
            if let Ok(client) = Aria2RpcClient::new(instance.port, self.config.secret.clone()) {
                println!("尝试优雅关闭 aria2...");
                if client.shutdown().await.is_ok() {
                    // 等待进程退出
                    let start_time = Instant::now();
                    while instance.is_running() && start_time.elapsed() < self.config.graceful_shutdown_timeout {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }

            // 如果优雅关闭失败，强制终止
            if instance.is_running() {
                println!("强制终止 aria2 进程");
                let _ = instance.kill();
            }
        }

        self.instance = None;
        println!("aria2 守护进程已停止");
    }

    pub async fn restart(&mut self) -> Result<(), DaemonError> {
        self.stop().await;
        self.restart_count = 0;  // 重置重启计数
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.start(&self.config.aria2_path).await
    }
}
```

## 监控和统计

### 运行状态监控
```rust
#[derive(Debug, Clone)]
pub struct DaemonStatus {
    pub is_running: bool,
    pub uptime: Duration,
    pub restart_count: u32,
    pub last_restart_time: Option<SystemTime>,
    pub current_port: Option<u16>,
    pub rpc_health: bool,
}

impl Aria2Daemon {
    pub fn get_status(&self) -> DaemonStatus {
        DaemonStatus {
            is_running: self.is_running.load(Ordering::SeqCst),
            uptime: self.start_time.map(|t| t.elapsed()).unwrap_or_default(),
            restart_count: self.restart_count,
            last_restart_time: self.last_restart_time,
            current_port: self.instance.as_ref().map(|i| i.port),
            rpc_health: self.last_health_check_success,
        }
    }

    pub async fn get_detailed_status(&self) -> DetailedStatus {
        let mut status = DetailedStatus {
            daemon: self.get_status(),
            aria2_version: None,
            global_stat: None,
            active_downloads: 0,
        };

        if let Some(ref instance) = self.instance {
            if let Ok(client) = Aria2RpcClient::new(instance.port, self.config.secret.clone()) {
                status.aria2_version = client.get_version().await.ok();
                status.global_stat = client.get_global_stat().await.ok();
                if let Ok(active) = client.tell_active(None).await {
                    status.active_downloads = active.len();
                }
            }
        }

        status
    }
}
```

### 日志记录
```rust
impl Aria2Daemon {
    fn log_event(&self, event: DaemonEvent) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match event {
            DaemonEvent::Started => {
                println!("[{}] 守护进程已启动", timestamp);
            }
            DaemonEvent::Stopped => {
                println!("[{}] 守护进程已停止", timestamp);
            }
            DaemonEvent::Restarted { count, reason } => {
                println!("[{}] aria2 已重启 (第{}次): {}", timestamp, count, reason);
            }
            DaemonEvent::HealthCheckFailed => {
                println!("[{}] 健康检查失败", timestamp);
            }
            DaemonEvent::MaxRestartsReached => {
                println!("[{}] 达到最大重启次数", timestamp);
            }
        }
    }
}

#[derive(Debug)]
enum DaemonEvent {
    Started,
    Stopped,
    Restarted { count: u32, reason: String },
    HealthCheckFailed,
    MaxRestartsReached,
}
```

## 错误处理

### 守护进程错误类型
```rust
#[derive(Debug)]
pub enum DaemonError {
    AlreadyRunning,
    NotRunning,
    StartupFailed(String),
    RestartLimitReached,
    ConfigError(String),
    ProcessError(std::io::Error),
    RpcError(RpcError),
}

impl std::fmt::Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonError::AlreadyRunning => write!(f, "守护进程已在运行"),
            DaemonError::NotRunning => write!(f, "守护进程未运行"),
            DaemonError::StartupFailed(msg) => write!(f, "启动失败: {}", msg),
            DaemonError::RestartLimitReached => write!(f, "达到最大重启次数"),
            DaemonError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
            DaemonError::ProcessError(e) => write!(f, "进程错误: {}", e),
            DaemonError::RpcError(e) => write!(f, "RPC错误: {:?}", e),
        }
    }
}
```

## 使用示例

### 基础守护进程使用
```rust
use crate::{Aria2Daemon, DaemonConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DaemonConfig::default();
    let mut daemon = Aria2Daemon::new(config);

    // 启动守护进程
    daemon.start(Path::new(r"C:\Users\username\AppData\Local\BurnCloud\aria2c.exe")).await?;

    println!("守护进程已启动，按 Ctrl+C 退出");

    // 等待中断信号
    tokio::signal::ctrl_c().await?;

    // 优雅关闭
    daemon.stop().await;

    Ok(())
}
```

### 高级监控使用
```rust
// 启动带监控的守护进程
let mut daemon = Aria2Daemon::new(DaemonConfig::default());
daemon.start(aria2_path).await?;

// 定期检查状态
loop {
    let status = daemon.get_detailed_status().await;

    println!("守护进程状态:");
    println!("  运行中: {}", status.daemon.is_running);
    println!("  运行时间: {:?}", status.daemon.uptime);
    println!("  重启次数: {}", status.daemon.restart_count);

    if let Some(port) = status.daemon.current_port {
        println!("  当前端口: {}", port);
    }

    if let Some(global_stat) = status.global_stat {
        println!("  活跃下载: {}", global_stat.num_active);
        println!("  下载速度: {}", global_stat.download_speed);
    }

    tokio::time::sleep(Duration::from_secs(10)).await;
}
```

## 配置建议

### 生产环境配置
```rust
let daemon_config = DaemonConfig {
    check_interval: Duration::from_secs(5),           // 5秒检查一次
    max_restarts: 50,                                 // 允许50次重启
    restart_delay: Duration::from_secs(10),           // 重启间隔10秒
    health_check_timeout: Duration::from_secs(5),     // 健康检查5秒超时
    graceful_shutdown_timeout: Duration::from_secs(60), // 优雅关闭60秒超时
};
```

### 开发环境配置
```rust
let daemon_config = DaemonConfig {
    check_interval: Duration::from_secs(2),           // 更频繁的检查
    max_restarts: 5,                                  // 较少的重启次数
    restart_delay: Duration::from_secs(1),            // 快速重启
    health_check_timeout: Duration::from_secs(3),     // 较短超时
    graceful_shutdown_timeout: Duration::from_secs(10), // 快速关闭
};
```

## 注意事项

1. **资源管理**: 确保正确清理子进程和文件句柄
2. **错误恢复**: 实现渐进式退避重启策略
3. **监控指标**: 记录关键指标用于故障排查
4. **信号处理**: 正确处理系统信号实现优雅关闭
5. **权限管理**: 确保有足够权限管理进程生命周期

## 测试要点

1. 进程意外退出的重启测试
2. RPC 服务无响应的检测和恢复
3. 达到最大重启次数的处理
4. 优雅关闭功能测试
5. 并发健康检查测试
6. 长时间运行稳定性测试