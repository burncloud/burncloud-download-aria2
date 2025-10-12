# poller/mod.rs - 进度轮询器模块文档

## 概述

`poller/mod.rs` 模块提供了下载进度的后台轮询功能。虽然当前实现中进度查询主要通过实时RPC调用完成，但该模块提供了轮询基础设施，可用于其他后台任务。

## 模块结构

### 子模块

- `aggregator` - 进度聚合模块，处理多文件下载的进度计算

## 主要结构

### ProgressPoller

后台进度轮询器，负责定期执行后台任务。

#### 字段

- `client: Arc<Aria2Client>` - aria2 RPC客户端的共享引用
- `shutdown: Arc<tokio::sync::Notify>` - 用于优雅关闭的通知机制

#### 特点

- **异步架构**: 使用tokio异步运行时
- **优雅关闭**: 支持通过Notify机制优雅停止
- **线程安全**: 使用Arc共享状态，支持多线程环境

## 构造方法

### new(client: Arc<Aria2Client>) -> Self

创建新的进度轮询器实例。

**参数**:
- `client`: aria2 RPC客户端的共享引用

**返回**: ProgressPoller实例

**示例**:
```rust
let client = Arc::new(Aria2Client::new(url, secret));
let poller = ProgressPoller::new(client);
```

## 公共方法

### start(&self)

启动后台轮询任务。

**功能**:
- 创建1秒间隔的定时器
- 在后台tokio任务中运行轮询循环
- 支持通过shutdown信号优雅停止

**实现细节**:
```rust
pub fn start(&self) {
    let _client = self.client.clone();
    let shutdown = self.shutdown.clone();

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    // 当前进度轮询通过实时RPC调用处理
                    // 此轮询器可用于其他后台任务
                }
                _ = shutdown.notified() => {
                    break;
                }
            }
        }
    });
}
```

**特性**:
- **非阻塞**: 在独立的tokio任务中运行
- **定时执行**: 每秒触发一次
- **可中断**: 响应shutdown信号立即停止

### shutdown(&self)

请求停止轮询器。

**功能**:
- 发送shutdown通知
- 触发轮询循环的优雅退出

**实现**:
```rust
pub fn shutdown(&self) {
    self.shutdown.notify_one();
}
```

## Drop Trait 实现

### drop(&mut self)

当ProgressPoller被销毁时自动调用shutdown。

**目的**: 确保资源清理和优雅关闭

**实现**:
```rust
impl Drop for ProgressPoller {
    fn drop(&mut self) {
        self.shutdown.notify_one();
    }
}
```

## 设计模式

### 1. 后台任务管理

使用tokio::spawn创建独立的异步任务：
- 不阻塞主线程
- 支持并发执行
- 自动资源管理

### 2. 优雅关闭模式

使用tokio::sync::Notify实现优雅关闭：
- 立即响应关闭请求
- 避免强制终止导致的资源泄漏
- 支持多种关闭触发方式

### 3. 选择性执行

使用tokio::select!宏实现条件执行：
- 定时器触发或关闭信号
- 避免忙等待
- 响应及时

## 使用示例

### 基本使用

```rust
use crate::poller::ProgressPoller;
use crate::client::Aria2Client;
use std::sync::Arc;

// 创建客户端和轮询器
let client = Arc::new(Aria2Client::new(
    "http://localhost:6800/jsonrpc".to_string(),
    Some("secret".to_string())
));
let poller = ProgressPoller::new(client);

// 启动后台轮询
poller.start();

// 执行其他任务...

// 优雅停止轮询器
poller.shutdown();
```

### 集成到下载管理器

```rust
use crate::manager::Aria2DownloadManager;

impl Aria2DownloadManager {
    pub async fn new(rpc_url: String, secret: Option<String>) -> Result<Self> {
        let client = Arc::new(Aria2Client::new(rpc_url, secret));

        // 创建并启动轮询器
        let poller = Arc::new(ProgressPoller::new(client.clone()));
        poller.start();

        Ok(Self {
            client,
            _poller: poller, // 保持引用，确保不被销毁
            // 其他字段...
        })
    }
}
```

## 扩展可能性

### 自定义轮询逻辑

可以扩展轮询器执行更复杂的后台任务：

```rust
tokio::select! {
    _ = ticker.tick() => {
        // 执行自定义后台任务
        self.perform_background_tasks().await;
    }
    _ = shutdown.notified() => {
        break;
    }
}
```

### 可配置间隔

支持自定义轮询间隔：

```rust
pub fn new_with_interval(client: Arc<Aria2Client>, interval: Duration) -> Self {
    // 实现支持自定义间隔的构造函数
}
```

### 任务调度

支持多种类型的定时任务：

```rust
struct ScheduledTask {
    interval: Duration,
    last_run: Instant,
    task: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>,
}
```

## 当前状态

目前的实现相对简化，主要原因：

1. **实时查询**: 进度信息通过实时RPC调用获取
2. **按需获取**: 只在需要时查询状态，减少不必要的网络请求
3. **简化架构**: 避免复杂的状态同步和缓存逻辑

## 性能考虑

### 资源使用

- **CPU**: 轻量级，每秒只有一次简单检查
- **内存**: 最小化状态，只保存必要的引用
- **网络**: 当前版本不执行网络请求

### 扩展性

- **多实例**: 支持多个轮询器并发运行
- **任务隔离**: 每个轮询器在独立的tokio任务中运行
- **资源共享**: 通过Arc共享客户端，避免重复连接

## 依赖关系

- `crate::client::Aria2Client` - aria2 RPC客户端
- `std::sync::Arc` - 引用计数智能指针
- `tokio::time::{interval, Duration}` - 异步定时器
- `tokio::sync::Notify` - 异步通知机制