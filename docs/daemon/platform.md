# daemon/platform.rs - 平台相关功能文档

## 概述

`daemon/platform.rs` 模块处理不同操作系统的平台差异，提供统一的接口来处理文件路径、权限和目录操作。

## 平台相关函数

### 二进制目录管理

#### get_binary_dir() -> PathBuf

获取平台特定的aria2二进制文件存储目录。

**Windows平台** (`#[cfg(target_os = "windows")]`)
- **路径**: `%LOCALAPPDATA%\BurnCloud`
- **默认**: `C:\Users\Default\AppData\Local\BurnCloud`
- **环境变量**: 使用 `LOCALAPPDATA` 环境变量

**Linux平台** (`#[cfg(target_os = "linux")]`)
- **路径**: `~/.burncloud`
- **后备路径**: `/tmp/.burncloud` (如果无法获取家目录)

**其他平台** (`#[cfg(not(any(target_os = "windows", target_os = "linux")))]`)
- **路径**: `~/.burncloud`
- **后备路径**: `/tmp/.burncloud` (如果无法获取家目录)

#### get_binary_name() -> &'static str

获取平台特定的aria2可执行文件名。

**Windows平台**
- **返回**: `"aria2c.exe"`

**非Windows平台** (`#[cfg(not(target_os = "windows"))]`)
- **返回**: `"aria2c"`

#### get_binary_path() -> PathBuf

获取aria2二进制文件的完整路径。

- **实现**: 结合 `get_binary_dir()` 和 `get_binary_name()`
- **返回**: 完整的二进制文件路径

### 目录和权限管理

#### ensure_directory(path: &Path) -> Result<(), Aria2Error>

确保指定目录存在，如果不存在则创建。

- **参数**: `path` - 需要确保存在的目录路径
- **功能**:
  - 检查目录是否存在
  - 如果不存在，递归创建目录
- **返回**: 成功或Aria2Error
- **异步**: 使用 `tokio::fs::create_dir_all`

#### set_executable(path: &Path) -> Result<(), Aria2Error>

设置文件的可执行权限（Unix系统特定）。

**Unix系统** (`#[cfg(unix)]`)
- **功能**:
  - 读取当前文件权限
  - 设置权限为 `0o755` (rwxr-xr-x)
  - 应用新权限
- **依赖**: `std::os::unix::fs::PermissionsExt`
- **异步**: 使用 `tokio::fs` 操作

**非Unix系统** (`#[cfg(not(unix))]`)
- **功能**: 空操作 (No-op)
- **原因**: Windows系统不需要显式设置可执行权限

## 设计特点

### 1. 条件编译

使用Rust的条件编译特性处理平台差异：
- `#[cfg(target_os = "windows")]` - Windows特定代码
- `#[cfg(target_os = "linux")]` - Linux特定代码
- `#[cfg(unix)]` - Unix-like系统特定代码
- `#[cfg(not(...))]` - 其他平台的默认实现

### 2. 环境变量处理

- 安全地处理环境变量（使用 `unwrap_or_else` 提供默认值）
- 遵循各平台的标准目录约定

### 3. 异步支持

- 所有文件系统操作都是异步的
- 使用 `tokio::fs` 而不是 `std::fs`

### 4. 错误处理

- 统一的错误类型 `Aria2Error`
- 自动转换标准库IO错误

## 使用示例

```rust
use crate::daemon::platform;

// 获取二进制文件路径
let binary_path = platform::get_binary_path();
println!("Aria2 binary path: {:?}", binary_path);

// 确保目录存在
let download_dir = PathBuf::from("/downloads");
platform::ensure_directory(&download_dir).await?;

// 设置可执行权限（Unix系统）
platform::set_executable(&binary_path).await?;

// 平台特定信息
println!("Binary name: {}", platform::get_binary_name());
println!("Binary directory: {:?}", platform::get_binary_dir());
```

## 平台差异总结

| 功能 | Windows | Linux/Unix | 其他平台 |
|------|---------|------------|----------|
| 二进制目录 | `%LOCALAPPDATA%\BurnCloud` | `~/.burncloud` | `~/.burncloud` |
| 二进制文件名 | `aria2c.exe` | `aria2c` | `aria2c` |
| 可执行权限 | 不需要 | 设置0o755 | 设置0o755 |
| 目录创建 | 支持 | 支持 | 支持 |

## 依赖关系

- `std::path::{Path, PathBuf}` - 路径处理
- `crate::error::Aria2Error` - 错误类型
- `dirs` crate - 家目录获取（Linux/其他平台）
- `tokio::fs` - 异步文件系统操作
- `std::os::unix::fs::PermissionsExt` - Unix权限处理