# BurnCloud Aria2 下载库

这是一个简单的 Rust 库，用于下载、配置和管理 aria2 下载器。

## 功能特性

### 1. aria2 二进制文件下载
- **主链接**: `https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip`
- **备用链接**: `https://gitee.com/burncloud/aria2/raw/master/aria2-1.37.0-win-64bit-build1.zip`
- 当主链接无法下载时，自动切换到备用链接

### 2. 端口管理
- 启动 RPC 服务前检查端口是否被占用
- 如果端口被占用，自动递增端口号直到找到可用端口
- 默认从端口 6800 开始检查

### 3. RPC 接口支持
实现完整的 aria2 RPC 接口功能，包括：
- 添加下载任务
- 暂停/恢复下载
- 获取下载状态
- 移除下载任务
- 获取全局统计信息
- 等等...

### 4. 守护进程功能
- 启动 aria2 后持续监控其运行状态
- 如果 aria2 意外退出，自动重启
- 提供健康检查功能

### 5. 设计原则
- **极度简单**: 代码结构清晰，易于理解和维护
- **单文件实现**: 所有功能都在 `lib.rs` 中实现
- **无复杂依赖**: 尽量使用标准库和必要的依赖
- **错误处理**: 简单但有效的错误处理机制

## 使用方式

```rust
use burncloud_download_aria2::*;

// 初始化并启动 aria2
let aria2 = Aria2Manager::new();
aria2.download_and_setup().await?;
aria2.start_daemon().await?;

// 使用 RPC 接口
aria2.add_download("http://example.com/file.zip").await?;
let status = aria2.get_global_stat().await?;
```

## 项目结构

```
burncloud-download-aria2/
├── src/
│   └── lib.rs                      # 主要实现文件
├── docs/
│   ├── README.md                   # 项目概述和使用指南
│   ├── aria2-download.md          # Aria2 下载功能详细文档
│   ├── port-management.md         # 端口管理和 RPC 启动文档
│   ├── rpc-interfaces.md          # RPC 接口功能文档
│   ├── daemon-process.md          # 守护进程文档
│   └── technical-specification.md  # 技术规范和架构设计
└── Cargo.toml                     # 项目配置
```

## 详细文档

### 功能模块文档
- **[Aria2 下载功能](aria2-download.md)** - 主备链接下载、故障转移机制
- **[端口管理和 RPC 启动](port-management.md)** - 端口检查、自动分配、RPC 服务启动
- **[RPC 接口功能](rpc-interfaces.md)** - 完整的 aria2 RPC 接口实现
- **[守护进程](daemon-process.md)** - 进程监控、自动重启、健康检查

### 技术文档
- **[技术规范和架构设计](technical-specification.md)** - 项目架构、设计原则、编码规范

## 依赖项

- `tokio`: 异步运行时
- `reqwest`: HTTP 客户端（用于下载 aria2）
- `serde_json`: JSON 处理（RPC 通信）
- `zip`: ZIP 文件解压

## 实现状态

- [ ] aria2 下载功能
- [ ] 端口检查和自动递增
- [ ] RPC 接口实现
- [ ] 守护进程功能
- [ ] 完整测试覆盖