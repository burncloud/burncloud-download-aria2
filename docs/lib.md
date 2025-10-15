# 库入口点 (lib.rs)

## 概述

`lib.rs` 是 BurnCloud Aria2 Download Manager 的主要入口点，提供公共 API 和便利函数，使用户能够轻松创建和使用 Aria2 下载管理器。

## 公共模块

### 导出的模块
- `client` - JSON-RPC 客户端实现
- `manager` - 下载管理器实现
- `poller` - 进度轮询器
- `error` - 错误定义
- `daemon` - 守护进程管理

### 主要导出类型
- `Aria2DownloadManager` - 主要的下载管理器类
- `Aria2Error` - 错误类型
- `Aria2Daemon` - 守护进程类
- `DaemonConfig` - 守护进程配置

## 常量定义

### `DEFAULT_ARIA2_RPC_URL`
```rust
pub const DEFAULT_ARIA2_RPC_URL: &str = "http://localhost:6800/jsonrpc";
```
- **用途**: 默认的 Aria2 RPC 端点
- **值**: `"http://localhost:6800/jsonrpc"`
- **说明**: 标准的 Aria2 JSON-RPC 服务地址

### `DEFAULT_ARIA2_SECRET`
```rust
pub const DEFAULT_ARIA2_SECRET: &str = "burncloud";
```
- **用途**: 默认的 Aria2 RPC 认证令牌
- **值**: `"burncloud"`
- **说明**: 用于 RPC 通信的安全令牌

## 便利函数

### `create_manager_with_auto_port()`

```rust
pub async fn create_manager_with_auto_port() -> anyhow::Result<Aria2DownloadManager>
```

**功能**: 创建带自动端口冲突解决的 Aria2DownloadManager

**特性**:
- 自动端口冲突检测和解决
- 从端口 6800 开始查找可用端口
- 如果端口被占用，自动递增寻找下一个可用端口
- 使用默认的 RPC 密钥 "burncloud"

**返回值**:
- `Ok(Aria2DownloadManager)` - 成功创建的管理器实例
- `Err(anyhow::Error)` - 创建失败的错误信息

**使用场景**:
- 快速启动，不需要自定义配置
- 自动处理多实例冲突
- 开发和测试环境

**示例**:
```rust
use burncloud_download_aria2::create_manager_with_auto_port;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_manager_with_auto_port().await?;
    println!("管理器创建成功，可以开始下载");
    Ok(())
}
```

### `create_manager_with_secret()`

```rust
pub async fn create_manager_with_secret(secret: &str) -> anyhow::Result<Aria2DownloadManager>
```

**功能**: 创建带自定义密钥和自动端口解决的 Aria2DownloadManager

**参数**:
- `secret: &str` - 自定义的 RPC 认证令牌

**特性**:
- 支持自定义 RPC 认证令牌
- 保持自动端口冲突解决功能
- 增强安全性

**返回值**:
- `Ok(Aria2DownloadManager)` - 成功创建的管理器实例
- `Err(anyhow::Error)` - 创建失败的错误信息

**使用场景**:
- 生产环境，需要自定义安全令牌
- 多租户系统，不同用户使用不同令牌
- 安全要求较高的环境

**示例**:
```rust
use burncloud_download_aria2::create_manager_with_secret;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_manager_with_secret("my_secure_token_123").await?;
    println!("管理器创建成功，使用自定义令牌");
    Ok(())
}
```

## 内部实现细节

### 自动端口解决机制

两个便利函数都内部调用 `Aria2DownloadManager::new()`，该方法会：

1. **端口检测**: 检查默认端口 6800 是否可用
2. **冲突解决**: 如果端口被占用，自动寻找下一个可用端口
3. **配置更新**: 更新 RPC URL 以使用新找到的端口
4. **守护进程启动**: 使用最终确定的端口启动 Aria2 守护进程
5. **健康检查**: 验证守护进程是否正常启动并可响应 RPC 请求

### 错误处理

便利函数使用 `anyhow::Result` 类型，可以包装各种底层错误：

- **端口冲突**: 如果找不到可用端口
- **守护进程启动失败**: 二进制文件问题或系统资源不足
- **RPC通信失败**: 网络或协议问题
- **配置错误**: 无效的参数或路径

### 异步设计

所有便利函数都是异步的，因为它们需要：
- 网络通信检查端口可用性
- 启动外部进程 (Aria2 守护进程)
- 建立 RPC 连接并进行健康检查

## 最佳实践

### 开发环境
```rust
// 快速开始，使用默认配置
let manager = create_manager_with_auto_port().await?;
```

### 生产环境
```rust
// 使用强密码
let manager = create_manager_with_secret("complex_secure_password_2024").await?;
```

### 错误处理
```rust
match create_manager_with_auto_port().await {
    Ok(manager) => {
        // 使用管理器
    },
    Err(e) => {
        eprintln!("创建管理器失败: {}", e);
        // 处理错误，可能的解决方案：
        // 1. 检查网络连接
        // 2. 确认有足够的系统权限
        // 3. 检查端口范围是否可用
    }
}
```

## 相关文档

- [错误处理](./error.md) - 详细的错误类型说明
- [管理器模块](./manager/) - 下载管理器的详细实现
- [守护进程模块](./daemon/) - 守护进程管理和配置