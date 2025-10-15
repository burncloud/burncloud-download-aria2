# 端口检查和 RPC 启动文档

## 概述

本模块负责检查端口占用情况，自动分配可用端口，并启动 aria2 的 RPC 服务。

## 端口管理策略

### 默认端口配置
- **起始端口**: 6800（aria2 默认 RPC 端口）
- **检查范围**: 6800-6900（最多检查 100 个端口）
- **递增策略**: 如果端口被占用，自动 +1 继续检查

### 端口检查流程
```
开始端口检查
    ↓
当前端口 = 6800
    ↓
检查端口是否可用
    ↓
可用？ ——— 是 ——→ 返回可用端口
    ↓ 否
端口 += 1
    ↓
端口 > 6900？ ——— 是 ——→ 返回错误
    ↓ 否
重新检查端口
```

## 实现要求

### 端口检查函数
```rust
pub fn check_port_available(port: u16) -> bool {
    // 尝试绑定端口
    match std::net::TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,   // 端口可用
        Err(_) => false, // 端口被占用
    }
}

pub fn find_available_port() -> Result<u16, PortError> {
    for port in 6800..=6900 {
        if check_port_available(port) {
            return Ok(port);
        }
    }
    Err(PortError::NoAvailablePort)
}
```

### 错误类型定义
```rust
#[derive(Debug)]
pub enum PortError {
    NoAvailablePort,
    BindError(String),
    PermissionDenied,
}
```

## RPC 服务启动

### 启动参数配置
```rust
pub struct Aria2Config {
    pub port: u16,
    pub secret: Option<String>,
    pub download_dir: PathBuf,
    pub max_connections: u8,
    pub split_size: String,
}

impl Default for Aria2Config {
    fn default() -> Self {
        Self {
            port: 6800,
            secret: None,
            download_dir: std::env::current_dir().unwrap().join("downloads"),
            max_connections: 16,
            split_size: "1M".to_string(),
        }
    }
}
```

### 启动命令构建
```rust
pub fn build_aria2_command(config: &Aria2Config, aria2_path: &Path) -> std::process::Command {
    let mut cmd = std::process::Command::new(aria2_path);

    cmd.args([
        "--enable-rpc",
        "--rpc-listen-all",
        &format!("--rpc-listen-port={}", config.port),
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

    cmd
}
```

### RPC 服务启动函数
```rust
pub async fn start_aria2_rpc(aria2_path: &Path) -> Result<Aria2Instance, StartupError> {
    // 1. 查找可用端口
    let port = find_available_port()?;

    // 2. 创建配置
    let mut config = Aria2Config::default();
    config.port = port;

    // 3. 构建启动命令
    let mut cmd = build_aria2_command(&config, aria2_path);

    // 4. 启动进程
    let child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // 5. 等待服务启动
    wait_for_rpc_ready(port).await?;

    Ok(Aria2Instance {
        process: child,
        port,
        config,
    })
}
```

## RPC 连接验证

### 健康检查
```rust
pub async fn wait_for_rpc_ready(port: u16) -> Result<(), StartupError> {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{}/jsonrpc", port);

    // 最多等待 30 秒
    for _ in 0..30 {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "test",
            "method": "aria2.getVersion"
        });

        match client.post(&url).json(&request).send().await {
            Ok(response) if response.status().is_success() => {
                return Ok(());
            }
            _ => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }

    Err(StartupError::RpcNotReady)
}
```

## Aria2 实例管理

### 实例结构
```rust
pub struct Aria2Instance {
    pub process: std::process::Child,
    pub port: u16,
    pub config: Aria2Config,
}

impl Aria2Instance {
    pub fn is_running(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(None) => true,      // 进程仍在运行
            Ok(Some(_)) => false,  // 进程已退出
            Err(_) => false,       // 错误，假设已退出
        }
    }

    pub fn kill(&mut self) -> Result<(), std::io::Error> {
        self.process.kill()?;
        self.process.wait()?;
        Ok(())
    }

    pub fn get_rpc_url(&self) -> String {
        format!("http://localhost:{}/jsonrpc", self.port)
    }
}
```

## 错误处理

### 启动错误类型
```rust
#[derive(Debug)]
pub enum StartupError {
    PortError(PortError),
    ProcessError(std::io::Error),
    RpcNotReady,
    ConfigError(String),
}
```

## 配置选项详解

### 核心参数
- `--enable-rpc`: 启用 JSON-RPC 接口
- `--rpc-listen-all`: 允许所有地址连接（生产环境建议限制）
- `--rpc-listen-port`: RPC 监听端口
- `--daemon=true`: 以守护进程模式运行

### 下载优化参数
- `--max-connection-per-server`: 每个服务器的最大连接数
- `--split`: 分片数量
- `--min-split-size`: 最小分片大小
- `--continue=true`: 支持断点续传
- `--max-tries=0`: 无限重试
- `--retry-wait=3`: 重试间隔（秒）

## 使用示例

```rust
use crate::{download_aria2, start_aria2_rpc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 下载 aria2
    let aria2_path = download_aria2().await?;

    // 2. 启动 RPC 服务
    let mut aria2 = start_aria2_rpc(&aria2_path).await?;

    println!("aria2 RPC 服务已启动在端口: {}", aria2.port);
    println!("RPC URL: {}", aria2.get_rpc_url());

    // 3. 保持运行状态
    loop {
        if !aria2.is_running() {
            eprintln!("aria2 进程意外退出");
            break;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}
```

## 注意事项

1. **权限要求**: 确保有权限绑定端口和启动进程
2. **防火墙**: 可能需要配置防火墙规则
3. **端口冲突**: 系统会自动处理端口冲突
4. **进程管理**: 确保正确清理子进程
5. **错误恢复**: 实现适当的错误恢复机制

## 测试要点

1. 端口占用检测准确性
2. 端口自动递增功能
3. RPC 服务启动成功验证
4. 配置参数正确传递
5. 进程状态监控
6. 错误情况处理