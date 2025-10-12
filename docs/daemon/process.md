# Process Management Module (daemon/process.rs)

## 概述

`daemon/process.rs` 模块负责 aria2 进程的生命周期管理，包括进程启动、停止、状态监控和重启计数。该模块提供了线程安全的进程管理接口，支持进程的优雅启停和异常重启机制。

## 结构体定义

### ProcessConfig
```rust
#[derive(Clone)]
pub struct ProcessConfig {
    pub rpc_port: u16,
    pub rpc_secret: String,
    pub download_dir: PathBuf,
    pub session_file: PathBuf,
    pub max_restart_attempts: u32,
}
```

**作用**: aria2 进程的配置结构体

**字段说明**:
- `rpc_port: u16` - RPC 服务监听端口（默认 6800）
- `rpc_secret: String` - RPC 访问密钥，用于身份验证
- `download_dir: PathBuf` - 下载文件存储目录
- `session_file: PathBuf` - 会话文件路径，用于恢复未完成的下载
- `max_restart_attempts: u32` - 最大重启尝试次数

**设计特点**:
- 实现 `Clone` trait，支持配置的克隆传递
- 所有字段都是公共的，便于外部模块配置

### ProcessHandle
```rust
pub struct ProcessHandle {
    child: Arc<Mutex<Option<Child>>>,
    restart_count: Arc<Mutex<u32>>,
    binary_path: PathBuf,
    config: ProcessConfig,
}
```

**作用**: aria2 进程的管理句柄，提供进程生命周期控制

**字段说明**:
- `child: Arc<Mutex<Option<Child>>>` - 子进程句柄，使用 Arc+Mutex 实现线程安全
- `restart_count: Arc<Mutex<u32>>` - 重启计数器，跟踪重启次数
- `binary_path: PathBuf` - aria2 二进制文件路径
- `config: ProcessConfig` - 进程配置信息

**线程安全设计**:
- 使用 `Arc<Mutex<>>` 包装可变状态
- 支持多线程环境下的安全访问
- 避免数据竞争和状态不一致

## 方法实现

### 构造方法

#### new
```rust
pub fn new(binary_path: PathBuf, config: ProcessConfig) -> Self
```

**作用**: 创建新的进程管理句柄

**参数**:
- `binary_path: PathBuf` - aria2 二进制文件的完整路径
- `config: ProcessConfig` - 进程配置参数

**返回值**:
- `Self` - 新的 ProcessHandle 实例

**初始状态**:
- 进程句柄为 `None`（未启动）
- 重启计数为 0
- 配置和二进制路径已设置

**使用示例**:
```rust
use std::path::PathBuf;
use crate::daemon::process::{ProcessHandle, ProcessConfig};

let config = ProcessConfig {
    rpc_port: 6800,
    rpc_secret: "my_secret".to_string(),
    download_dir: PathBuf::from("/downloads"),
    session_file: PathBuf::from("/config/aria2.session"),
    max_restart_attempts: 5,
};

let binary_path = PathBuf::from("/usr/bin/aria2c");
let handle = ProcessHandle::new(binary_path, config);
```

### 进程控制方法

#### start_process
```rust
pub async fn start_process(&self) -> Result<(), Aria2Error>
```

**作用**: 启动 aria2 进程

**返回值**:
- `Result<(), Aria2Error>` - 成功返回 `Ok(())`，失败返回错误

**执行流程**:
1. **停止现有进程**: 如果已有进程在运行，先停止它
2. **创建目录**: 确保下载目录和会话文件目录存在
3. **构建命令**: 设置 aria2c 启动参数
4. **配置输出**: 根据构建类型配置日志输出
5. **启动进程**: 创建子进程并存储句柄

**命令行参数**:
```bash
aria2c --enable-rpc \
       --rpc-listen-port 6800 \
       --rpc-secret "secret" \
       --dir "/downloads" \
       --continue \
       --save-session "/config/aria2.session" \
       --save-session-interval 60 \
       [--input-file "/config/aria2.session"]  # 仅当会话文件存在时
       [--quiet]  # 仅在 release 构建时
```

**调试支持**:
- Debug 构建时保留进程输出用于诊断
- Release 构建时使用 `--quiet` 参数并重定向输出到 `/dev/null`
- 捕获和打印 stderr 输出用于故障排除

**错误处理**:
- 目录创建失败
- 进程启动失败
- 配置参数无效

**使用示例**:
```rust
match handle.start_process().await {
    Ok(()) => println!("aria2 started successfully"),
    Err(e) => eprintln!("Failed to start aria2: {}", e),
}
```

#### stop_process
```rust
pub async fn stop_process(&self) -> Result<(), Aria2Error>
```

**作用**: 优雅地停止 aria2 进程

**返回值**:
- `Result<(), Aria2Error>` - 成功返回 `Ok(())`，失败返回错误

**执行流程**:
1. **获取进程锁**: 安全地访问子进程句柄
2. **发送终止信号**: 调用 `child.kill()` 终止进程
3. **等待退出**: 调用 `child.wait()` 等待进程完全退出
4. **清理状态**: 将进程句柄设置为 `None`

**错误处理**:
- 忽略 `InvalidInput` 错误（进程已退出）
- 其他错误转换为 `ProcessManagementError`

**使用示例**:
```rust
match handle.stop_process().await {
    Ok(()) => println!("aria2 stopped successfully"),
    Err(e) => eprintln!("Failed to stop aria2: {}", e),
}
```

### 状态查询方法

#### is_running
```rust
pub async fn is_running(&self) -> bool
```

**作用**: 检查 aria2 进程是否正在运行

**返回值**:
- `bool` - 进程运行中返回 `true`，否则返回 `false`

**检查逻辑**:
1. **获取进程锁**: 安全地访问子进程句柄
2. **状态查询**: 使用 `try_wait()` 非阻塞检查进程状态
3. **状态更新**: 如果进程已退出，清理句柄
4. **返回结果**: 根据进程状态返回布尔值

**状态判断**:
- `Ok(Some(_))` - 进程已退出
- `Ok(None)` - 进程仍在运行
- `Err(_)` - 查询失败，假定进程不在运行

**使用示例**:
```rust
if handle.is_running().await {
    println!("aria2 is running");
} else {
    println!("aria2 is not running");
}
```

### 重启管理方法

#### increment_restart_count
```rust
pub async fn increment_restart_count(&self) -> u32
```

**作用**: 增加重启计数器并返回新值

**返回值**:
- `u32` - 增加后的重启次数

**线程安全**:
- 使用 Mutex 保护计数器访问
- 原子性地增加计数值

**使用场景**:
- 进程崩溃时记录重启尝试
- 达到最大重启次数时停止重启

**使用示例**:
```rust
let restart_count = handle.increment_restart_count().await;
if restart_count > handle.max_restart_attempts() {
    eprintln!("Max restart attempts exceeded");
}
```

#### reset_restart_count
```rust
pub async fn reset_restart_count(&self)
```

**作用**: 重置重启计数器为 0

**使用场景**:
- 进程成功运行一段时间后
- 健康检查通过时
- 手动重置重启状态

**使用示例**:
```rust
// 进程健康运行后重置计数
if process_healthy {
    handle.reset_restart_count().await;
}
```

#### max_restart_attempts
```rust
pub fn max_restart_attempts(&self) -> u32
```

**作用**: 获取配置的最大重启尝试次数

**返回值**:
- `u32` - 最大重启次数

**特点**:
- 同步方法，无需 await
- 只读访问配置信息

## 配置详解

### RPC 配置
- **端口选择**: 默认 6800，可自定义避免冲突
- **安全认证**: 通过 `rpc_secret` 保护 RPC 接口
- **网络绑定**: 监听本地端口，支持远程访问

### 文件管理
- **下载目录**: 可配置的下载文件存储位置
- **会话文件**: 保存下载状态，支持断点续传
- **自动恢复**: 启动时自动加载会话文件

### 进程选项
- **持续下载**: `--continue` 支持断点续传
- **会话保存**: 定期保存下载状态（60秒间隔）
- **静默模式**: Release 构建时启用，减少日志输出

## 调试和诊断

### Debug 构建特性
```rust
#[cfg(debug_assertions)]
{
    eprintln!("DEBUG: Starting aria2c with command:");
    eprintln!("  Binary: {:?}", self.binary_path);
    eprintln!("  Args: {:?}", cmd);
}
```

- 打印完整的启动命令
- 捕获并显示 stderr 输出
- 保留进程输出用于故障排除

### 错误日志
- 实时打印 aria2 的错误输出
- 帮助诊断启动失败原因
- 在后台任务中异步处理日志

## 依赖关系

### 内部依赖
- `crate::error::Aria2Error` - 错误类型定义

### 外部依赖
- `std::path::PathBuf` - 路径操作
- `std::sync::Arc` - 原子引用计数
- `tokio::sync::Mutex` - 异步互斥锁
- `tokio::process::Command` - 异步进程管理
- `std::process::Stdio` - 标准输入输出配置

## 设计模式

### 1. 资源管理模式
- 使用 RAII 自动管理进程生命周期
- 安全的资源获取和释放

### 2. 状态封装模式
- 内部状态通过 Mutex 保护
- 提供安全的状态访问接口

### 3. 错误处理模式
- 详细的错误分类和上下文
- 优雅的错误传播和处理

### 4. 异步设计模式
- 所有 I/O 操作都是异步的
- 支持并发操作和非阻塞执行

## 使用注意事项

1. **权限要求**: 需要执行二进制文件和创建目录的权限
2. **端口冲突**: 确保 RPC 端口未被其他进程占用
3. **磁盘空间**: 确保下载目录有足够的空间
4. **网络配置**: RPC 端口需要在防火墙中开放（如需远程访问）
5. **并发安全**: 多个线程可以安全地访问同一个 ProcessHandle

## 相关模块

- [`binary.rs`](binary.md) - 提供二进制文件下载和管理
- [`monitor.rs`](monitor.md) - 使用 ProcessHandle 进行健康监控
- [`orchestrator.rs`](orchestrator.md) - 协调进程管理和整体控制