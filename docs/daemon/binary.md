# Binary Management Module (daemon/binary.rs)

## 概述

`daemon/binary.rs` 模块负责管理 aria2 二进制文件的下载、验证和解压缩。该模块提供了自动化的二进制文件管理功能，支持多个下载源（GitHub 和 Gitee）的故障转移机制。

## 常量定义

### GITHUB_DOWNLOAD_URL
```rust
const GITHUB_DOWNLOAD_URL: &str = "https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip";
```
- **类型**: `&str`
- **作用**: 主要下载源 - GitHub 官方发布地址
- **版本**: aria2 1.37.0 Windows 64位版本

### GITEE_DOWNLOAD_URL
```rust
const GITEE_DOWNLOAD_URL: &str = "https://gitee.com/burncloud/aria2/raw/master/aria2-1.37.0-win-64bit-build1.zip";
```
- **类型**: `&str`
- **作用**: 备用下载源 - Gitee 镜像地址
- **用途**: 当 GitHub 下载失败时的故障转移选项

## 公共函数

### verify_binary_exists
```rust
pub async fn verify_binary_exists(path: &Path) -> bool
```

**作用**: 验证指定路径的二进制文件是否存在

**参数**:
- `path: &Path` - 要检查的文件路径

**返回值**:
- `bool` - 文件存在返回 `true`，否则返回 `false`

**实现原理**:
- 使用 `tokio::fs::metadata()` 异步检查文件元数据
- 通过 `is_ok()` 判断文件是否可访问

**使用示例**:
```rust
use std::path::Path;
use crate::daemon::binary::verify_binary_exists;

let binary_path = Path::new("/path/to/aria2c.exe");
if verify_binary_exists(&binary_path).await {
    println!("Binary exists!");
} else {
    println!("Binary not found, need to download");
}
```

### download_aria2_binary
```rust
pub async fn download_aria2_binary(target_path: &Path) -> Result<(), Aria2Error>
```

**作用**: 下载并安装 aria2 二进制文件到指定路径

**参数**:
- `target_path: &Path` - 二进制文件的目标安装路径

**返回值**:
- `Result<(), Aria2Error>` - 成功返回 `Ok(())`，失败返回相应错误

**执行流程**:
1. 确保父目录存在（调用 `platform::ensure_directory()`）
2. 尝试从 GitHub 下载
3. 如果 GitHub 失败，从 Gitee 备用源下载
4. 解压缩 ZIP 文件并提取 aria2c 二进制文件
5. 在 Unix 系统上设置可执行权限

**错误处理**:
- 网络下载失败会尝试备用源
- 所有源都失败时返回 `BinaryDownloadFailed` 错误
- 解压失败返回 `BinaryExtractionFailed` 错误

**使用示例**:
```rust
use std::path::Path;
use crate::daemon::binary::download_aria2_binary;

let install_path = Path::new("/usr/local/bin/aria2c");
match download_aria2_binary(&install_path).await {
    Ok(()) => println!("Binary downloaded successfully"),
    Err(e) => eprintln!("Download failed: {}", e),
}
```

## 私有函数

### download_from_url
```rust
async fn download_from_url(url: &str) -> Result<Vec<u8>, Aria2Error>
```

**作用**: 从指定 URL 下载文件数据

**参数**:
- `url: &str` - 下载地址

**返回值**:
- `Result<Vec<u8>, Aria2Error>` - 成功返回文件字节数据，失败返回错误

**实现细节**:
- 使用 `reqwest::get()` 发起 HTTP 请求
- 检查 HTTP 状态码，只接受成功响应
- 将响应体转换为字节向量

**错误情况**:
- 网络请求失败
- HTTP 状态码表示错误
- 读取响应体失败

### extract_zip
```rust
async fn extract_zip(zip_data: Vec<u8>, target_path: &Path) -> Result<(), Aria2Error>
```

**作用**: 从 ZIP 数据中提取 aria2c 二进制文件

**参数**:
- `zip_data: Vec<u8>` - ZIP 文件的字节数据
- `target_path: &Path` - 提取目标路径

**返回值**:
- `Result<(), Aria2Error>` - 成功返回 `Ok(())`，失败返回错误

**实现逻辑**:
1. 创建内存中的 ZIP 读取器（`Cursor` + `ZipArchive`）
2. 获取平台特定的二进制文件名（通过 `platform::get_binary_name()`）
3. 遍历 ZIP 条目，查找匹配的二进制文件
4. 找到后提取到目标路径

**平台兼容性**:
- Windows: 查找 `aria2c.exe`
- Unix/Linux: 查找 `aria2c`

**错误处理**:
- ZIP 文件格式错误
- 目标文件创建失败
- 文件拷贝失败
- 二进制文件在 ZIP 中不存在

## 依赖关系

### 内部依赖
- `crate::error::Aria2Error` - 错误类型定义
- `super::platform` - 平台特定功能模块

### 外部依赖
- `std::path::Path` - 路径操作
- `std::io::Cursor` - 内存中的数据流
- `reqwest` - HTTP 客户端（隐式使用）
- `zip` - ZIP 文件解压缩（隐式使用）

## 设计特点

### 1. 故障转移机制
模块实现了双源下载策略：
- 优先使用 GitHub 官方源
- GitHub 失败时自动切换到 Gitee 镜像
- 提高了下载成功率和用户体验

### 2. 异步设计
所有 I/O 操作都是异步的：
- 文件存在性检查
- 网络下载
- 文件解压缩和写入

### 3. 平台无关性
通过 `platform` 模块抽象了平台特定功能：
- 自动识别二进制文件名
- 处理目录创建
- 设置可执行权限

### 4. 错误处理
详细的错误信息和类型化错误处理：
- 网络错误包含具体的失败原因
- 解压错误提供文件操作上下文
- 支持错误链追踪

## 使用注意事项

1. **网络环境**: 需要能够访问 GitHub 或 Gitee
2. **磁盘空间**: 确保目标位置有足够的磁盘空间
3. **权限**: 需要在目标目录的写入权限
4. **并发安全**: 函数是并发安全的，但不建议同时下载到同一路径

## 相关模块

- [`platform.rs`](platform.md) - 提供平台特定的实用功能
- [`process.rs`](process.md) - 使用下载的二进制文件启动进程
- [`orchestrator.rs`](orchestrator.md) - 协调二进制下载和进程管理