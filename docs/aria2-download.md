# Aria2 下载功能文档

## 概述

本模块负责从指定的链接下载 aria2 二进制文件，支持主链接和备用链接的故障转移机制。

## 下载链接配置

### 主链接（官方 GitHub Release）
```
https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip
```

### 备用链接（国内镜像）
```
https://gitee.com/burncloud/aria2/raw/master/aria2-1.37.0-win-64bit-build1.zip
```

## 下载策略

### 1. 故障转移机制
- 优先使用主链接进行下载
- 如果主链接下载失败（网络错误、超时、404等），自动切换到备用链接
- 确保在网络环境受限的情况下仍能成功下载

### 2. 下载流程
```
开始下载
    ↓
尝试主链接下载
    ↓
下载成功？ ——— 是 ——→ 返回文件路径
    ↓ 否
尝试备用链接下载
    ↓
下载成功？ ——— 是 ——→ 返回文件路径
    ↓ 否
返回下载失败错误
```

## 实现要求

### 函数签名
```rust
pub async fn download_aria2() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 实现下载逻辑
}
```

### 关键功能
1. **HTTP 客户端配置**
   - 设置合理的超时时间（建议 30 秒）
   - 支持重定向
   - 设置用户代理

2. **文件管理**
   - 下载到临时目录
   - 验证下载完整性（文件大小检查）
   - 创建必要的目录结构

3. **错误处理**
   - 网络连接错误
   - HTTP 状态码错误
   - 文件写入错误
   - 磁盘空间不足

### 错误类型
```rust
#[derive(Debug)]
pub enum DownloadError {
    NetworkError(String),
    HttpError(u16),
    FileError(String),
    BothLinksFailedError,
}
```

## 文件解压处理

### ZIP 解压
- 下载完成后自动解压 ZIP 文件
- 提取 `aria2c.exe` 到指定目录
- 清理临时 ZIP 文件

### 目录结构
```
./aria2/
├── aria2c.exe          # 主要可执行文件
├── README.txt          # 说明文件
└── COPYING             # 许可证文件
```

## 依赖库

### 必需依赖
- `reqwest`: HTTP 客户端，用于文件下载
- `tokio`: 异步运行时
- `zip`: ZIP 文件解压缩

### 配置示例
```toml
[dependencies]
reqwest = { version = "0.11", features = ["stream"] }
tokio = { version = "1.0", features = ["full"] }
zip = "0.6"
```

## 使用示例

```rust
use crate::download_aria2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 下载 aria2
    let aria2_path = download_aria2().await?;
    println!("aria2 已下载到: {:?}", aria2_path);

    Ok(())
}
```

## 注意事项

1. **网络环境**：考虑到不同网络环境的限制，备用链接使用国内镜像
2. **版本固定**：使用固定版本（1.37.0）确保兼容性
3. **平台特定**：当前仅支持 Windows 64 位版本
4. **权限要求**：确保有足够权限写入目标目录
5. **磁盘空间**：至少需要 10MB 可用空间

## 测试要点

1. 主链接正常下载
2. 主链接失败时备用链接下载
3. 两个链接都失败的错误处理
4. 网络超时处理
5. 文件完整性验证
6. ZIP 解压功能