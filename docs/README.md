# BurnCloud Aria2 Download Manager Documentation

## 项目概述

BurnCloud Aria2 Download Manager 是一个基于 Rust 的 Aria2 下载管理器实现，为 BurnCloud 项目提供下载后端支持。

## 核心特性

- 自定义 JSON-RPC 客户端，与 aria2 进行通信
- 支持 HTTP/HTTPS、BitTorrent、Metalink 和 Magnet 下载
- 每秒轮询进度更新
- 多文件下载聚合进度报告
- 完整的 DownloadManager trait 实现
- 自动端口冲突解决 - 从端口 6800 开始，如果被占用则自动递增

## 架构概览

项目采用模块化设计，主要包含以下模块：

### 🔧 核心模块
- **lib.rs** - 库的主入口点，提供公共 API
- **error.rs** - 错误定义和处理

### 📡 客户端模块 (client/)
- **mod.rs** - Aria2 JSON-RPC 客户端实现
- **types.rs** - 客户端相关的数据类型定义

### 🎛️ 管理器模块 (manager/)
- **mod.rs** - 下载管理器的主要实现

### 🔄 轮询器模块 (poller/)
- **mod.rs** - 进度轮询器
- **aggregator.rs** - 进度聚合器

### 🚀 守护进程模块 (daemon/)
- **mod.rs** - 守护进程模块定义
- **orchestrator.rs** - 守护进程主协调器
- **process.rs** - 进程生命周期管理
- **monitor.rs** - 健康状况监控
- **binary.rs** - 二进制文件管理
- **platform.rs** - 平台相关操作
- **port_utils.rs** - 端口工具函数

## 文档结构

- [错误处理](./error.md) - 错误类型和处理策略
- [库入口](./lib.md) - 主要公共 API 说明
- [客户端模块](./client/) - JSON-RPC 客户端文档
- [管理器模块](./manager/) - 下载管理器文档
- [轮询器模块](./poller/) - 进度轮询文档
- [守护进程模块](./daemon/) - 守护进程文档

## 快速开始

```rust
use burncloud_download_aria2::create_manager_with_auto_port;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建下载管理器，自动处理端口冲突
    let manager = create_manager_with_auto_port().await?;

    // 管理器现在可以使用了
    Ok(())
}
```

## 系统架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    BurnCloud Download Manager                │
├─────────────────────────────────────────────────────────────┤
│  Public API (lib.rs)                                       │
│  ┌─────────────────┐ ┌─────────────┐ ┌─────────────────────┐│
│  │ Manager         │ │ Client      │ │ Daemon              ││
│  │                 │ │             │ │                     ││
│  │ - Task Mapping  │ │ - JSON-RPC  │ │ - Process Mgmt     ││
│  │ - Progress      │ │ - Transport │ │ - Health Monitor   ││
│  │ - State Mgmt    │ │ - Protocol  │ │ - Binary Mgmt      ││
│  └─────────────────┘ └─────────────┘ └─────────────────────┘│
│           │                 │                     │         │
│           │                 │                     │         │
│           └─────────────────┼─────────────────────┘         │
│                             │                               │
│  ┌─────────────────────────┐│┌─────────────────────────────┐│
│  │ Poller                  │││ Platform Utils              ││
│  │                         │││                             ││
│  │ - Progress Polling      │││ - Port Management           ││
│  │ - Aggregation          │││ - File Operations           ││
│  └─────────────────────────┘│└─────────────────────────────┘│
│                             │                               │
├─────────────────────────────┼─────────────────────────────────┤
│                             ▼                               │
│                    Aria2 Binary Process                     │
│                    (JSON-RPC Server)                        │
└─────────────────────────────────────────────────────────────┘
```