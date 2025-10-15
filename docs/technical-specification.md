# 技术规范和架构设计文档

## 项目概述

BurnCloud Aria2 下载库是一个极简的 Rust 库，专注于下载、管理和守护 aria2 下载器。遵循"极度简单"的设计原则，将所有功能集中在单个 `lib.rs` 文件中实现。

## 设计原则

### 1. 极度简单 (Extreme Simplicity)
- **单文件架构**: 所有功能都在 `lib.rs` 中实现
- **最小依赖**: 仅使用必要的外部依赖
- **清晰接口**: 提供简洁明了的 API
- **直接实现**: 避免过度抽象和复杂的设计模式

### 2. 功能完整性 (Complete Functionality)
- **完整的 RPC 接口**: 实现所有 aria2 RPC 方法
- **自动故障转移**: 主备链接下载机制
- **智能端口管理**: 自动检测和分配可用端口
- **进程守护**: 自动重启和健康监控

### 3. 可靠性优先 (Reliability First)
- **错误处理**: 全面的错误处理和恢复机制
- **资源管理**: 正确的资源生命周期管理
- **进程监控**: 持续的健康检查和自动恢复

## 技术架构

### 整体架构图
```
┌─────────────────────────────────────────────────────────────┐
│                    BurnCloud Aria2 Library                 │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   Aria2Manager  │  │ Aria2RpcClient  │  │ Aria2Daemon  │ │
│  │                 │  │                 │  │              │ │
│  │ - download()    │  │ - add_uri()     │  │ - start()    │ │
│  │ - setup()       │  │ - tell_status() │  │ - monitor()  │ │
│  │ - start_daemon()│  │ - pause()       │  │ - restart()  │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │  Port Manager   │  │ Process Manager │  │ Error Types  │ │
│  │                 │  │                 │  │              │ │
│  │ - check_port()  │  │ - start_aria2() │  │ - RpcError   │ │
│  │ - find_port()   │  │ - monitor()     │  │ - DaemonErr  │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                        External Dependencies                │
│  tokio | reqwest | serde_json | zip | base64              │
└─────────────────────────────────────────────────────────────┘
```

### 核心模块结构
```rust
// lib.rs 的模块组织
pub struct Aria2Manager {
    // 主管理器 - 统一入口点
}

pub struct Aria2RpcClient {
    // RPC 客户端 - 所有 aria2 接口
}

pub struct Aria2Daemon {
    // 守护进程 - 进程监控和重启
}

pub struct Aria2Instance {
    // aria2 实例 - 进程封装
}

// 支持结构和枚举
pub struct Aria2Config { /* ... */ }
pub struct DownloadOptions { /* ... */ }
pub struct DownloadStatus { /* ... */ }
pub enum RpcError { /* ... */ }
pub enum DaemonError { /* ... */ }
```

## API 设计规范

### 1. 统一的入口点
```rust
pub struct Aria2Manager {
    // 提供最简单的使用接口
}

impl Aria2Manager {
    pub fn new() -> Self;
    pub async fn download_and_setup(&mut self) -> Result<(), Box<dyn Error>>;
    pub async fn start_daemon(&mut self) -> Result<(), Box<dyn Error>>;
    pub fn get_rpc_client(&self) -> Option<&Aria2RpcClient>;
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn Error>>;
}
```

### 2. 异步接口设计
- 所有 I/O 操作都使用 `async/await`
- 返回类型使用 `Result<T, E>` 进行错误处理
- 支持并发操作和取消

### 3. 错误处理策略
```rust
// 统一的错误类型
pub type Aria2Result<T> = Result<T, Aria2Error>;

#[derive(Debug)]
pub enum Aria2Error {
    DownloadError(String),
    PortError(String),
    RpcError(String),
    DaemonError(String),
    ConfigError(String),
}
```

## 实现规范

### 1. 代码组织
```rust
// lib.rs 文件结构
#![deny(unsafe_code)]
#![warn(missing_docs)]

// 外部依赖导入
use std::...;
use tokio::...;
use reqwest::...;
use serde::{Deserialize, Serialize};

// 常量定义
const DEFAULT_PORT: u16 = 6800;
const MAX_PORT_RANGE: u16 = 100;
const HEALTH_CHECK_INTERVAL: u64 = 5;

// 公开类型定义
pub struct Aria2Manager { ... }
pub struct Aria2RpcClient { ... }
pub struct Aria2Daemon { ... }

// 实现块
impl Aria2Manager { ... }
impl Aria2RpcClient { ... }
impl Aria2Daemon { ... }

// 辅助函数
async fn download_aria2() -> Result<PathBuf, DownloadError> { ... }
fn find_available_port() -> Result<u16, PortError> { ... }
async fn start_aria2_process(...) -> Result<Aria2Instance, ProcessError> { ... }

// 错误类型定义
#[derive(Debug)]
pub enum Aria2Error { ... }
```

### 2. 编码规范
- **命名约定**: 使用 `snake_case` 命名函数和变量，`PascalCase` 命名类型
- **文档注释**: 所有公共 API 必须有文档注释
- **错误处理**: 使用 `Result` 类型，避免 `panic!`
- **资源管理**: 实现适当的 `Drop` trait
- **异步安全**: 确保多线程安全

### 3. 性能要求
- **内存使用**: 尽量减少内存分配和复制
- **网络效率**: 使用连接池，避免频繁建立连接
- **并发处理**: 支持多个并发下载任务
- **响应时间**: RPC 调用响应时间 < 100ms

## 依赖管理

### 核心依赖
```toml
[dependencies]
# 异步运行时
tokio = { version = "1.0", features = ["full"] }

# HTTP 客户端
reqwest = { version = "0.11", features = ["json", "stream"] }

# JSON 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# ZIP 文件处理
zip = "0.6"

# Base64 编码
base64 = "0.21"
```

### 开发依赖
```toml
[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.0"
wiremock = "0.5"
```

## 配置规范

### 默认配置
```rust
impl Default for Aria2Config {
    fn default() -> Self {
        Self {
            port: 6800,
            secret: None,
            download_dir: std::env::current_dir().unwrap().join("downloads"),
            max_connections: 16,
            split_size: "1M".to_string(),
            max_download_speed: None,
            max_upload_speed: None,
            user_agent: "BurnCloud-Aria2/1.0".to_string(),
        }
    }
}
```

### 环境变量支持
- `ARIA2_PORT`: 覆盖默认端口
- `ARIA2_SECRET`: 设置 RPC 密码
- `ARIA2_DOWNLOAD_DIR`: 设置下载目录
- `ARIA2_MAX_CONNECTIONS`: 设置最大连接数

## 测试策略

### 1. 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_download_aria2() {
        // 测试 aria2 下载功能
    }

    #[tokio::test]
    async fn test_port_management() {
        // 测试端口管理功能
    }

    #[tokio::test]
    async fn test_rpc_client() {
        // 测试 RPC 客户端
    }
}
```

### 2. 集成测试
```rust
// tests/integration_tests.rs
use burncloud_download_aria2::*;

#[tokio::test]
async fn test_full_workflow() {
    let mut manager = Aria2Manager::new();

    // 测试完整工作流程
    manager.download_and_setup().await.unwrap();
    manager.start_daemon().await.unwrap();

    let client = manager.get_rpc_client().unwrap();
    let gid = client.add_uri(vec!["http://example.com/test.zip".to_string()], None).await.unwrap();

    // 验证下载状态
    let status = client.tell_status(&gid, None).await.unwrap();
    assert_eq!(status.status, "active");

    manager.shutdown().await.unwrap();
}
```

### 3. 性能测试
- 并发下载测试
- 长时间运行稳定性测试
- 内存泄漏测试
- 网络异常恢复测试

## 部署和发布

### 1. 版本管理
- 使用语义化版本 (SemVer)
- 维护 CHANGELOG.md
- Git 标签对应版本号

### 2. 文档维护
- API 文档自动生成 (`cargo doc`)
- 示例代码保持更新
- 用户指南和最佳实践

### 3. 持续集成
```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

## 安全考虑

### 1. 网络安全
- HTTPS 下载验证
- RPC 接口访问控制
- 敏感信息加密存储

### 2. 进程安全
- 权限最小化
- 安全的进程清理
- 避免进程注入攻击

### 3. 输入验证
- URL 格式验证
- 文件路径安全检查
- RPC 参数验证

## 扩展性设计

### 1. 插件机制
```rust
pub trait DownloadPlugin {
    async fn pre_download(&self, url: &str) -> Result<(), PluginError>;
    async fn post_download(&self, result: &DownloadResult) -> Result<(), PluginError>;
}
```

### 2. 配置扩展
- 支持配置文件 (TOML/JSON)
- 运行时配置修改
- 配置验证和迁移

### 3. 监控集成
- 指标收集接口
- 日志结构化输出
- 健康检查端点

## 最佳实践

### 1. 使用建议
```rust
// 推荐的使用方式
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut aria2 = Aria2Manager::new();

    // 初始化和启动
    aria2.download_and_setup().await?;
    aria2.start_daemon().await?;

    // 使用 RPC 客户端
    let client = aria2.get_rpc_client().unwrap();
    let gid = client.add_uri(vec!["http://example.com/file.zip".to_string()], None).await?;

    // 监控下载进度
    loop {
        let status = client.tell_status(&gid, None).await?;
        if status.status == "complete" {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // 优雅关闭
    aria2.shutdown().await?;
    Ok(())
}
```

### 2. 错误处理模式
```rust
// 错误处理最佳实践
match client.add_uri(urls, options).await {
    Ok(gid) => {
        println!("下载任务已添加: {}", gid);
    }
    Err(RpcError::NetworkError(e)) => {
        eprintln!("网络错误，请检查连接: {}", e);
        // 实现重试逻辑
    }
    Err(RpcError::ServerError(e)) => {
        eprintln!("aria2 服务器错误: {:?}", e);
        // 尝试重启服务
    }
    Err(e) => {
        eprintln!("未知错误: {:?}", e);
        return Err(e.into());
    }
}
```

### 3. 资源管理
```rust
// 确保资源正确清理
impl Drop for Aria2Manager {
    fn drop(&mut self) {
        // 异步清理需要使用 runtime
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(async {
                let _ = self.shutdown().await;
            });
        }
    }
}
```

## 总结

本技术规范定义了 BurnCloud Aria2 下载库的完整架构和实现指南。遵循这些规范可以确保代码的简洁性、可靠性和可维护性。重点强调：

1. **极简设计**: 单文件实现，最小依赖
2. **完整功能**: 涵盖所有必需的 aria2 功能
3. **错误恢复**: 强大的错误处理和自动恢复机制
4. **易于使用**: 简洁的 API 和清晰的文档