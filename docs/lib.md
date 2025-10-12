# lib.rs - 库根模块文档

## 概述

`lib.rs` 是 BurnCloud Aria2 Download Manager 的主要库入口文件，它定义了整个库的模块结构和公共接口。

## 功能特性

- 自定义JSON-RPC客户端，用于与aria2通信
- 支持HTTP/HTTPS、BitTorrent、Metalink和Magnet下载
- 每秒进行进度轮询
- 多文件下载聚合进度报告
- 完整的DownloadManager trait实现

## 模块结构

### 导出的模块

- `client` - Aria2客户端模块，处理与aria2的通信
- `manager` - 下载管理器模块，实现核心下载管理功能
- `poller` - 轮询器模块，处理下载状态轮询
- `error` - 错误处理模块，定义错误类型
- `daemon` - 守护进程模块，管理aria2进程

### 公共接口

```rust
pub use manager::Aria2DownloadManager;
pub use error::Aria2Error;
pub use daemon::{Aria2Daemon, DaemonConfig};
```

#### 导出的结构和类型

1. **Aria2DownloadManager** - 主要的下载管理器实现
   - 位置：`src/manager/mod.rs`
   - 作用：提供统一的下载管理接口

2. **Aria2Error** - 错误类型定义
   - 位置：`src/error.rs`
   - 作用：处理库中的各种错误情况

3. **Aria2Daemon** - Aria2守护进程管理器
   - 位置：`src/daemon/mod.rs`
   - 作用：管理aria2进程的启动、停止和监控

4. **DaemonConfig** - 守护进程配置
   - 位置：`src/daemon/mod.rs`
   - 作用：配置aria2守护进程的运行参数

## 常量定义

### DEFAULT_ARIA2_RPC_URL
- **类型**: `&str`
- **值**: `"http://localhost:6800/jsonrpc"`
- **作用**: 默认的aria2 RPC端点地址

### DEFAULT_ARIA2_SECRET
- **类型**: `&str`
- **值**: `"burncloud"`
- **作用**: 默认的aria2 RPC密钥令牌

## 使用示例

```rust
use burncloud_download_aria2::{Aria2DownloadManager, Aria2Error, Aria2Daemon, DaemonConfig};

// 使用默认配置创建下载管理器
let manager = Aria2DownloadManager::new();

// 使用默认RPC地址
let rpc_url = burncloud_download_aria2::DEFAULT_ARIA2_RPC_URL;
```