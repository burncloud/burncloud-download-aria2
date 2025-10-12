# daemon/mod.rs - 守护进程模块入口文档

## 概述

`daemon/mod.rs` 是aria2守护进程管理模块的入口文件，定义了模块结构和公共接口。

## 模块结构

### 内部模块 (pub(crate))

- `platform` - 平台相关功能，处理不同操作系统的差异
- `binary` - 二进制文件管理，包括下载和验证aria2可执行文件
- `process` - 进程管理，处理aria2进程的启动、停止和监控
- `monitor` - 健康监控，实现aria2进程的健康检查和自动重启

### 私有模块

- `orchestrator` - 主要的协调器模块，组织各个子模块的功能

### 测试模块

- `tests` - 单元测试（仅在测试时编译）

## 公共接口

```rust
pub use orchestrator::{Aria2Daemon, DaemonConfig};
```

### 导出的类型

#### Aria2Daemon
- **位置**: `orchestrator.rs`
- **作用**: 主要的aria2守护进程管理器
- **功能**: 启动、停止和管理aria2守护进程

#### DaemonConfig
- **位置**: `orchestrator.rs`
- **作用**: 守护进程配置结构
- **功能**: 配置aria2守护进程的运行参数

## 模块间依赖关系

```
mod.rs
├── orchestrator (核心协调器)
│   ├── platform (平台相关)
│   ├── binary (二进制管理)
│   ├── process (进程管理)
│   └── monitor (健康监控)
└── tests (测试)
```

## 设计原则

1. **模块化**: 每个子模块专注于特定的功能领域
2. **平台抽象**: 通过platform模块处理操作系统差异
3. **责任分离**: 二进制管理、进程控制、健康监控分离
4. **统一接口**: 通过orchestrator提供统一的API

## 使用示例

```rust
use crate::daemon::{Aria2Daemon, DaemonConfig};
use crate::client::Aria2Client;
use std::sync::Arc;

// 创建配置
let config = DaemonConfig::default();

// 创建客户端
let client = Arc::new(Aria2Client::new(
    format!("http://localhost:{}/jsonrpc", config.rpc_port),
    Some(config.rpc_secret.clone())
));

// 启动守护进程
let daemon = Aria2Daemon::start(config, client).await?;

// 使用守护进程...

// 停止守护进程
daemon.stop().await?;
```