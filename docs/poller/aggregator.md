# poller/aggregator.rs - 进度聚合器模块文档

## 概述

`poller/aggregator.rs` 模块提供了多文件下载进度聚合功能。当下载任务包含多个文件时（如BitTorrent或Metalink下载），该模块能够计算整体的下载进度。

## 主要结构

### ProgressAggregator

静态进度聚合器，提供多文件下载进度的聚合计算。

#### 设计特点

- **无状态**: 纯静态方法，不保存状态
- **函数式**: 接收输入，返回计算结果
- **高效**: 最小化内存分配和计算开销

## 核心方法

### aggregate(status: &Aria2Status) -> AggregatedProgress

聚合多文件下载的进度信息。

#### 参数

- `status: &Aria2Status` - aria2返回的状态信息，包含文件列表

#### 返回值

- `AggregatedProgress` - 聚合后的进度信息

#### 聚合逻辑

1. **总大小计算**:
   - 遍历所有文件
   - 解析每个文件的`length`字段
   - 过滤无效值，求和得到总大小

2. **已下载大小计算**:
   - 遍历所有文件
   - 解析每个文件的`completed_length`字段
   - 过滤无效值，求和得到已下载大小

#### 实现细节

```rust
pub fn aggregate(status: &Aria2Status) -> AggregatedProgress {
    let total_bytes: u64 = status.files.iter()
        .filter_map(|f| f.length.parse::<u64>().ok())
        .sum();

    let downloaded_bytes: u64 = status.files.iter()
        .filter_map(|f| f.completed_length.parse::<u64>().ok())
        .sum();

    AggregatedProgress {
        total_bytes,
        downloaded_bytes,
    }
}
```

#### 错误处理

- **解析失败**: 使用`filter_map`和`parse().ok()`优雅处理无效数据
- **自动跳过**: 无法解析的字段自动被跳过，不会导致整体计算失败
- **零值处理**: 如果所有解析都失败，返回0值，确保程序稳定性

## 数据结构

### AggregatedProgress

聚合进度信息的结构体。

#### 字段

- `total_bytes: u64` - 总字节数
- `downloaded_bytes: u64` - 已下载字节数

#### 特性

- **Debug**: 支持调试输出
- **Clone**: 支持克隆操作
- **简单**: 只包含基本的进度信息

#### 衍生计算

虽然结构体本身只包含基本信息，但可以轻松计算其他指标：

```rust
impl AggregatedProgress {
    pub fn progress_percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }

    pub fn remaining_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.downloaded_bytes)
    }

    pub fn is_complete(&self) -> bool {
        self.total_bytes > 0 && self.downloaded_bytes >= self.total_bytes
    }
}
```

## 使用场景

### 1. BitTorrent 下载

BitTorrent下载通常包含多个文件：
```rust
// aria2状态包含多个.files条目
let torrent_status = client.tell_status("torrent_gid").await?;
let progress = ProgressAggregator::aggregate(&torrent_status);
println!("整体进度: {}/{} bytes", progress.downloaded_bytes, progress.total_bytes);
```

### 2. Metalink 下载

Metalink文件可以指定多个镜像和文件：
```rust
let metalink_status = client.tell_status("metalink_gid").await?;
let progress = ProgressAggregator::aggregate(&metalink_status);
```

### 3. 单文件下载

对于单文件下载，聚合器仍然有效：
```rust
let single_file_status = client.tell_status("http_gid").await?;
let progress = ProgressAggregator::aggregate(&single_file_status);
// 结果与单个文件的进度相同
```

## 测试模块

### test_aggregate_multi_file

测试多文件下载的进度聚合功能。

#### 测试数据

创建包含两个文件的模拟状态：
- 文件1: 1000字节总大小，500字节已完成
- 文件2: 2000字节总大小，1000字节已完成

#### 预期结果

- 总大小: 3000字节 (1000 + 2000)
- 已下载: 1500字节 (500 + 1000)

#### 测试代码

```rust
#[test]
fn test_aggregate_multi_file() {
    let status = Aria2Status {
        // ... 基本字段
        files: vec![
            Aria2File {
                length: "1000".to_string(),
                completed_length: "500".to_string(),
                // ... 其他字段
            },
            Aria2File {
                length: "2000".to_string(),
                completed_length: "1000".to_string(),
                // ... 其他字段
            },
        ],
        // ...
    };

    let progress = ProgressAggregator::aggregate(&status);
    assert_eq!(progress.total_bytes, 3000);
    assert_eq!(progress.downloaded_bytes, 1500);
}
```

## 设计考虑

### 1. 性能优化

- **迭代器链**: 使用惰性求值的迭代器，避免中间分配
- **单次遍历**: 虽然有两次循环，但每次都是O(n)，总体复杂度仍为O(n)
- **无状态**: 不保存状态，避免内存泄漏

### 2. 健壮性

- **容错**: 无效数据不会导致程序崩溃
- **一致性**: 总是返回有效的`AggregatedProgress`实例
- **可预测**: 相同输入总是产生相同输出

### 3. 扩展性

当前实现专注于基本聚合，但可以轻松扩展：

```rust
pub struct DetailedProgress {
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub file_count: usize,
    pub completed_files: usize,
    pub files: Vec<FileProgress>,
}

pub struct FileProgress {
    pub path: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub selected: bool,
}
```

## 实际应用

### 集成到下载管理器

```rust
impl Aria2DownloadManager {
    pub async fn get_detailed_progress(&self, task_id: TaskId) -> Result<AggregatedProgress> {
        let gid = self.get_gid_for_task(task_id)?;
        let status = self.client.tell_status(&gid).await?;
        Ok(ProgressAggregator::aggregate(&status))
    }
}
```

### 进度条显示

```rust
fn display_progress(progress: &AggregatedProgress) {
    let percentage = if progress.total_bytes > 0 {
        (progress.downloaded_bytes as f64 / progress.total_bytes as f64) * 100.0
    } else {
        0.0
    };

    println!("进度: {:.1}% ({}/{})",
        percentage,
        format_bytes(progress.downloaded_bytes),
        format_bytes(progress.total_bytes)
    );
}
```

## 局限性

### 1. 字符串解析

aria2返回的数值都是字符串格式，需要解析：
- 可能存在解析失败的情况
- 大数值可能超出u64范围

### 2. 精度问题

- 只支持字节级精度
- 不考虑部分下载的块

### 3. 静态信息

- 不包含动态信息如下载速度
- 需要结合其他数据源获取完整信息

## 未来增强

### 1. 更详细的聚合

```rust
pub struct EnhancedProgress {
    pub basic: AggregatedProgress,
    pub speed_bps: u64,
    pub eta_seconds: Option<u64>,
    pub active_files: usize,
}
```

### 2. 增量计算

支持基于历史数据的增量计算：
```rust
pub fn aggregate_delta(
    current: &Aria2Status,
    previous: &AggregatedProgress,
) -> ProgressDelta {
    // 计算增量变化
}
```

## 依赖关系

- `crate::client::types::{Aria2Status, Aria2File}` - aria2状态结构体