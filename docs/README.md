# BurnCloud Aria2 Download Manager - 代码文档

## 概述

本文档目录包含了 BurnCloud Aria2 Download Manager 项目的完整代码文档。该项目是一个基于 Rust 的 aria2 下载管理器，提供了完整的下载管理功能。

## 文档结构

```
docs/
├── README.md                    # 本文档（总览）
├── lib.md                       # 库根模块文档
├── error.md                     # 错误处理模块文档
├── client/                      # 客户端模块文档
│   ├── mod.md                   # 客户端主模块
│   └── types.md                 # 客户端类型定义
├── daemon/                      # 守护进程模块文档
│   ├── mod.md                   # 守护进程主模块
│   ├── platform.md             # 平台相关功能
│   ├── binary.md                # 二进制文件管理
│   ├── process.md               # 进程管理
│   ├── monitor.md               # 健康监控
│   ├── orchestrator.md          # 主协调器
│   └── README.md                # 守护进程模块总览
├── manager/                     # 下载管理器模块文档
│   ├── mod.md                   # 管理器主模块
│   └── mapper.md                # 状态映射
└── poller/                      # 轮询器模块文档
    ├── mod.md                   # 轮询器主模块
    └── aggregator.md            # 进度聚合器
```

## 核心模块介绍

### 🚀 [lib.rs](./lib.md) - 库入口

- 项目的主入口点
- 定义模块结构和公共接口
- 提供统一的导出和常量定义

### ❌ [error.rs](./error.md) - 错误处理

- 统一的错误类型定义
- 完整的错误分类和处理
- 与外部库的错误转换

### 🌐 [client/](./client/) - Aria2 客户端

- **[mod.rs](./client/mod.md)**: JSON-RPC 客户端实现
- **[types.rs](./client/types.md)**: 数据类型定义和序列化

**核心功能**:
- JSON-RPC 2.0 通信协议
- 支持所有 aria2 下载类型
- 完整的任务控制接口

### 🔧 [daemon/](./daemon/) - 守护进程管理

- **[orchestrator.rs](./daemon/orchestrator.md)**: 主协调器
- **[process.rs](./daemon/process.md)**: 进程生命周期管理
- **[monitor.rs](./daemon/monitor.md)**: 健康监控和自动重启
- **[binary.rs](./daemon/binary.md)**: 二进制文件下载和管理
- **[platform.rs](./daemon/platform.md)**: 跨平台兼容性

**核心功能**:
- 自动下载和管理 aria2 二进制文件
- 跨平台进程管理
- 健康监控和故障恢复
- 优雅的启动和关闭流程

### 📦 [manager/](./manager/) - 下载管理器

- **[mod.rs](./manager/mod.md)**: 主下载管理器实现
- **[mapper.rs](./manager/mapper.md)**: 状态映射逻辑

**核心功能**:
- 实现标准的 DownloadManager trait
- 支持多种下载协议
- 任务状态管理和转换
- 进度信息计算

### 📊 [poller/](./poller/) - 进度轮询

- **[mod.rs](./poller/mod.md)**: 后台轮询器
- **[aggregator.rs](./poller/aggregator.md)**: 多文件进度聚合

**核心功能**:
- 后台任务调度
- 多文件下载进度聚合
- 优雅的生命周期管理

## 架构概览

### 整体架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Application   │────│ DownloadManager │────│   Aria2Client   │
│    (用户层)      │    │   (管理层)       │    │   (通信层)       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │ ProgressPoller  │    │  Aria2Daemon    │
                       │   (监控层)       │    │   (进程层)       │
                       └─────────────────┘    └─────────────────┘
                                                        │
                                                        ▼
                                               ┌─────────────────┐
                                               │   aria2 进程    │
                                               │   (底层服务)     │
                                               └─────────────────┘
```

### 数据流

```
用户请求 → DownloadManager → TaskId 映射 → Aria2Client → JSON-RPC → aria2
              ↓                                                    ↑
        进度查询 ← ProgressPoller ← 状态映射 ← Aria2Status ←────────┘
```

## 特性和优势

### ✨ 核心特性

1. **完整的下载支持**
   - HTTP/HTTPS/FTP
   - BitTorrent
   - Metalink/Meta4
   - 磁力链接

2. **智能进程管理**
   - 自动下载 aria2 二进制文件
   - 健康监控和自动重启
   - 优雅启动和关闭

3. **跨平台兼容**
   - Windows、Linux、macOS 支持
   - 平台特定的路径和权限处理

4. **异步架构**
   - 基于 tokio 异步运行时
   - 非阻塞操作
   - 高并发支持

5. **强类型安全**
   - Rust 类型系统
   - 编译时错误检查
   - 内存安全保证

### 🎯 设计原则

1. **模块化设计**: 每个模块专注于特定功能
2. **责任分离**: 清晰的接口和职责划分
3. **错误处理**: 完整的错误分类和传播
4. **资源管理**: 自动的生命周期管理
5. **扩展性**: 易于添加新功能和协议

## 使用示例

### 基本使用

```rust
use burncloud_download_aria2::{Aria2DownloadManager, DaemonConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建下载管理器
    let manager = Aria2DownloadManager::new(
        "http://localhost:6800/jsonrpc".to_string(),
        Some("secret".to_string())
    ).await?;

    // 添加下载任务
    let task_id = manager.add_download(
        "http://example.com/file.zip".to_string(),
        PathBuf::from("/downloads/file.zip")
    ).await?;

    // 监控进度
    loop {
        let progress = manager.get_progress(task_id).await?;
        println!("进度: {}/{} bytes ({}%)",
            progress.downloaded_bytes,
            progress.total_bytes.unwrap_or(0),
            if let Some(total) = progress.total_bytes {
                (progress.downloaded_bytes * 100 / total)
            } else { 0 }
        );

        let task = manager.get_task(task_id).await?;
        if matches!(task.status, DownloadStatus::Completed) {
            println!("下载完成！");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
```

### 高级配置

```rust
use burncloud_download_aria2::{Aria2Daemon, DaemonConfig};
use std::time::Duration;

// 自定义守护进程配置
let config = DaemonConfig {
    rpc_port: 6801,
    rpc_secret: "my_custom_secret".to_string(),
    download_dir: PathBuf::from("/custom/downloads"),
    max_restart_attempts: 5,
    health_check_interval: Duration::from_secs(5),
    ..Default::default()
};

// 直接启动守护进程
let client = Arc::new(Aria2Client::new(
    format!("http://localhost:{}/jsonrpc", config.rpc_port),
    Some(config.rpc_secret.clone())
));

let daemon = Aria2Daemon::start(config, client).await?;
```

## 开发指南

### 添加新的下载协议

1. 在 `DownloadType` 枚举中添加新类型
2. 在 `detect_download_type` 中添加检测逻辑
3. 在 `add_download` 中添加处理分支
4. 可能需要扩展 aria2 客户端方法

### 扩展状态映射

1. 在 `mapper.rs` 中添加新的状态映射规则
2. 更新 `DownloadStatus` 枚举（如果需要）
3. 添加相应的测试用例

### 平台支持

1. 在 `platform.rs` 中添加平台特定代码
2. 使用条件编译标记 `#[cfg(...)]`
3. 更新二进制下载 URL 和路径逻辑

## 测试和调试

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test client::
cargo test daemon::
cargo test manager::
```

### 调试模式

项目在 debug 模式下提供额外的诊断信息：

- aria2 进程启动参数
- 错误详细信息
- 网络请求日志

### 日志输出

使用环境变量控制日志级别：

```bash
RUST_LOG=debug cargo run
```

## 依赖说明

### 核心依赖

- **tokio**: 异步运行时
- **reqwest**: HTTP 客户端
- **serde**: 序列化框架
- **anyhow**: 错误处理
- **uuid**: 唯一标识符生成

### 平台依赖

- **dirs**: 目录路径获取
- **zip**: 压缩文件处理
- **base64**: Base64 编码

## 性能考虑

### 内存使用

- 使用 `Arc` 共享数据，减少复制
- 流式处理大文件下载
- 及时清理不需要的状态

### 网络效率

- 连接复用
- 批量状态查询
- 智能轮询间隔

### 并发性能

- 异步操作避免阻塞
- 支持多任务并发下载
- 线程安全的状态管理

## 故障排除

### 常见问题

1. **aria2 启动失败**
   - 检查端口是否被占用
   - 验证下载目录权限
   - 查看 debug 模式输出

2. **RPC 连接失败**
   - 确认 aria2 进程运行状态
   - 检查防火墙设置
   - 验证 RPC 密钥配置

3. **下载任务不响应**
   - 检查网络连接
   - 验证 URL 有效性
   - 查看 aria2 错误日志

### 调试技巧

1. 使用 debug 编译获取详细日志
2. 检查 aria2 会话文件内容
3. 使用网络抓包分析 RPC 通信
4. 监控系统资源使用情况

## 贡献指南

### 代码风格

- 遵循 Rust 官方格式规范
- 使用 `cargo fmt` 格式化代码
- 通过 `cargo clippy` 检查

### 文档要求

- 为公共 API 添加文档注释
- 更新相关的 markdown 文档
- 包含使用示例

### 测试要求

- 为新功能添加单元测试
- 确保所有测试通过
- 添加集成测试（如适用）

---

## 总结

BurnCloud Aria2 Download Manager 是一个功能完整、设计良好的下载管理解决方案。通过模块化的架构、强类型的设计和完善的错误处理，它提供了可靠、高效的下载管理服务。

本文档集合为开发者提供了完整的代码理解和使用指南，有助于项目的维护、扩展和集成。