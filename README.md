# BurnCloud Aria2 下载库

这是一个极简的 Rust 库，用于下载、配置和管理 aria2 下载器。遵循"极度简单"的设计原则，所有功能都在单个 `lib.rs` 文件中实现。

## 特性

- ✅ **Aria2 自动下载**: 从 GitHub 或备用链接自动下载 aria2 二进制文件
- ✅ **智能端口管理**: 自动检测和分配可用端口（6800-6900）
- ✅ **RPC 接口**: 完整的 aria2 JSON-RPC 2.0 接口支持
- ✅ **简单守护进程**: 基础的进程管理功能
- ✅ **统一管理器**: 一站式管理接口

## 快速开始

### 添加依赖

```toml
[dependencies]
burncloud-download-aria2 = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### 基础使用

```rust
use burncloud_download_aria2::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 快速启动 aria2 管理器
    let mut manager = quick_start().await?;

    println!("✅ aria2 已启动！");

    // 创建 RPC 客户端
    if let Some(client) = manager.create_rpc_client() {
        // 添加下载任务
        let gid = client.add_uri(
            vec!["http://example.com/file.zip".to_string()],
            None
        ).await?;

        println!("📥 下载任务已添加: {}", gid);

        // 获取下载状态
        let status = client.tell_status(&gid).await?;
        println!("📊 状态: {}", status.status);

        // 获取全局统计
        let stats = client.get_global_stat().await?;
        println!("📈 活跃下载: {}", stats.num_active);
    }

    // 关闭管理器
    manager.shutdown().await?;
    Ok(())
}
```

## 手动配置

```rust
use burncloud_download_aria2::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 自定义配置
    let config = Aria2Config {
        port: 6800,
        secret: Some("my-secret".to_string()),
        download_dir: std::path::PathBuf::from("./downloads"),
        max_connections: 8,
        split_size: "1M".to_string(),
        aria2_path: std::path::PathBuf::from(r"C:\Users\username\AppData\Local\BurnCloud\aria2c.exe"),
    };

    let mut manager = Aria2Manager::with_config(config);

    // 下载和设置 aria2
    manager.download_and_setup().await?;

    // 启动守护进程
    manager.start_daemon().await?;

    // 使用 RPC 客户端...

    manager.shutdown().await?;
    Ok(())
}
```

## 运行示例

```bash
# 编译并运行示例
cargo run

# 或者只编译检查
cargo check
```

## API 文档

### 主要结构

- `Aria2Manager`: 统一管理器，提供最简单的使用接口
- `Aria2RpcClient`: RPC 客户端，用于与 aria2 通信
- `Aria2Daemon`: 简单守护进程，管理 aria2 进程生命周期

### 关键方法

#### Aria2Manager
- `new()`: 创建新管理器
- `download_and_setup()`: 下载并设置 aria2
- `start_daemon()`: 启动守护进程
- `create_rpc_client()`: 创建 RPC 客户端
- `shutdown()`: 关闭管理器

#### Aria2RpcClient
- `add_uri(uris, options)`: 添加下载任务
- `tell_status(gid)`: 获取下载状态
- `tell_active()`: 获取活跃下载列表
- `get_global_stat()`: 获取全局统计
- `pause(gid)`: 暂停下载
- `unpause(gid)`: 恢复下载
- `remove(gid)`: 移除下载

## 错误处理

所有方法都返回 `Aria2Result<T>`，统一的错误类型：

```rust
pub enum Aria2Error {
    DownloadError(String),
    PortError(String),
    RpcError(String),
    DaemonError(String),
    ProcessError(String),
    ConfigError(String),
}
```

## 系统要求

- Windows 10/11 (64位)
- Rust 1.70+
- 网络连接（用于下载 aria2）

## 文件位置

Aria2 二进制文件默认下载到：
```
C:\Users\username\AppData\Local\BurnCloud\aria2c.exe
```

## 注意事项

1. **首次运行**: 需要网络连接下载 aria2 二进制文件
2. **端口占用**: 程序会自动检测 6800-6900 端口范围内的可用端口
3. **权限要求**: 需要有权限在 AppData\Local 目录创建文件
4. **防火墙**: 可能需要允许 aria2c.exe 通过防火墙

## 许可证

MIT License - 详见 LICENSE 文件

## 贡献

欢迎提交 Issue 和 Pull Request！