# manager/mapper.rs - 状态映射模块文档

## 概述

`manager/mapper.rs` 模块负责将aria2的状态字符串映射为标准的`DownloadStatus`枚举类型。这个模块提供了aria2特定状态与通用下载状态之间的转换逻辑。

## 主要功能

### map_aria2_status(aria2_status: &Aria2Status) -> DownloadStatus

将aria2状态对象映射为标准的下载状态枚举。

#### 参数

- `aria2_status: &Aria2Status` - aria2返回的状态信息

#### 返回值

- `DownloadStatus` - 标准化的下载状态枚举

#### 状态映射规则

| aria2状态 | DownloadStatus | 说明 |
|-----------|----------------|------|
| "active" | Downloading | 正在下载 |
| "waiting" | Waiting | 等待开始下载 |
| "paused" | Paused | 已暂停 |
| "complete" | Completed | 下载完成 |
| "error" | Failed(error_msg) | 下载失败，包含错误信息 |
| "removed" | Failed("Download cancelled") | 下载已取消 |
| 其他 | Failed("Unknown status: ...") | 未知状态 |

#### 错误状态处理

对于 "error" 状态，函数会提取详细的错误信息：

1. **优先使用错误消息**: 如果 `aria2_status.error_message` 存在，直接使用
2. **使用错误代码**: 如果没有错误消息，使用 `aria2_status.error_code`
3. **默认消息**: 如果都不存在，使用 "unknown" 作为错误代码

```rust
let error_msg = aria2_status.error_message
    .as_ref()
    .map(|s| s.clone())
    .unwrap_or_else(|| format!("Error code: {}",
        aria2_status.error_code.as_ref().unwrap_or(&"unknown".to_string())));
```

#### 使用示例

```rust
use crate::manager::mapper::map_aria2_status;
use crate::client::types::Aria2Status;

// 创建aria2状态
let aria2_status = Aria2Status {
    gid: "123456".to_string(),
    status: "active".to_string(),
    total_length: "1048576".to_string(),
    completed_length: "524288".to_string(),
    download_speed: "102400".to_string(),
    upload_speed: "0".to_string(),
    files: vec![],
    error_code: None,
    error_message: None,
};

// 映射状态
let download_status = map_aria2_status(&aria2_status);
assert_eq!(download_status, DownloadStatus::Downloading);
```

## 测试模块

### 测试辅助函数

#### create_test_status(status: &str) -> Aria2Status

创建用于测试的Aria2Status实例。

**参数**:
- `status: &str` - 要设置的状态字符串

**返回**:
- 预配置的Aria2Status实例，包含测试用的默认值

### 测试用例

#### test_map_active_status

测试活动状态的映射。

```rust
#[test]
fn test_map_active_status() {
    let status = create_test_status("active");
    assert_eq!(map_aria2_status(&status), DownloadStatus::Downloading);
}
```

#### test_map_waiting_status

测试等待状态的映射。

```rust
#[test]
fn test_map_waiting_status() {
    let status = create_test_status("waiting");
    assert_eq!(map_aria2_status(&status), DownloadStatus::Waiting);
}
```

#### test_map_paused_status

测试暂停状态的映射。

```rust
#[test]
fn test_map_paused_status() {
    let status = create_test_status("paused");
    assert_eq!(map_aria2_status(&status), DownloadStatus::Paused);
}
```

#### test_map_complete_status

测试完成状态的映射。

```rust
#[test]
fn test_map_complete_status() {
    let status = create_test_status("complete");
    assert_eq!(map_aria2_status(&status), DownloadStatus::Completed);
}
```

#### test_map_error_status

测试错误状态的映射，包括错误消息的提取。

```rust
#[test]
fn test_map_error_status() {
    let mut status = create_test_status("error");
    status.error_message = Some("Connection failed".to_string());
    match map_aria2_status(&status) {
        DownloadStatus::Failed(msg) => assert_eq!(msg, "Connection failed"),
        _ => panic!("Expected Failed status"),
    }
}
```

## 设计特点

### 1. 状态标准化

- 将aria2特定的状态字符串转换为通用的枚举类型
- 提供一致的状态表示，便于上层应用处理

### 2. 错误信息保留

- 完整保留aria2提供的错误信息
- 支持错误代码和错误消息的组合
- 提供有意义的错误描述

### 3. 健壮性

- 处理未知状态，避免程序崩溃
- 优雅地处理缺失的错误信息
- 提供默认的错误描述

### 4. 测试覆盖

- 完整的单元测试覆盖所有状态映射
- 测试边界情况和错误处理
- 确保映射逻辑的正确性

## 状态转换图

```
aria2状态映射流程:

aria2 "active"    → DownloadStatus::Downloading
aria2 "waiting"   → DownloadStatus::Waiting
aria2 "paused"    → DownloadStatus::Paused
aria2 "complete"  → DownloadStatus::Completed
aria2 "error"     → DownloadStatus::Failed(详细错误信息)
aria2 "removed"   → DownloadStatus::Failed("Download cancelled")
其他状态          → DownloadStatus::Failed("Unknown status: ...")
```

## 扩展性

### 添加新状态映射

如果aria2引入新的状态类型，可以通过修改匹配模式来支持：

```rust
pub fn map_aria2_status(aria2_status: &Aria2Status) -> DownloadStatus {
    match aria2_status.status.as_str() {
        "active" => DownloadStatus::Downloading,
        "waiting" => DownloadStatus::Waiting,
        "paused" => DownloadStatus::Paused,
        "complete" => DownloadStatus::Completed,
        "new_status" => DownloadStatus::NewStatus, // 新增状态
        "error" => {
            // 错误处理逻辑
        }
        "removed" => DownloadStatus::Failed("Download cancelled".to_string()),
        _ => DownloadStatus::Failed(format!("Unknown status: {}", aria2_status.status)),
    }
}
```

### 自定义错误处理

可以扩展错误信息的处理逻辑，支持更复杂的错误分类：

```rust
"error" => {
    let error_code = aria2_status.error_code.as_ref();
    let error_msg = aria2_status.error_message.as_ref();

    // 根据错误代码进行分类处理
    match error_code.map(|s| s.as_str()) {
        Some("1") => DownloadStatus::Failed("Network error".to_string()),
        Some("2") => DownloadStatus::Failed("File not found".to_string()),
        _ => {
            // 默认错误处理
        }
    }
}
```

## 依赖关系

- `burncloud_download_types::DownloadStatus` - 标准下载状态枚举
- `crate::client::types::Aria2Status` - aria2状态结构体