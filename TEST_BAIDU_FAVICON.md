# 百度 Favicon 下载测试

## 测试说明

这个测试用例验证 aria2 daemon 可以成功下载真实的网络文件。

**测试文件**: `tests/daemon_integration_test.rs::test_download_baidu_favicon`

**下载位置**: 文件会保存到 `burncloud-download-aria2/test_downloads/baidu_favicon.ico`（不会自动删除）

**测试内容**:
- 下载 https://www.baidu.com/favicon.ico
- 监控下载进度
- 验证文件完整性
- 验证 ICO 文件格式

## ⚠️ 重要提示

**文件保存位置**: 下载的文件会保存在项目目录下的 `test_downloads/` 文件夹中，测试结束后**不会自动删除**，方便你查看下载结果。

## 运行测试

### 前置条件

1. **网络连接**: 需要能够访问 www.baidu.com
2. **aria2 二进制**: 测试会自动下载（如果不存在）
3. **端口可用**: 确保端口 6800 未被占用

### 运行命令

```bash
# 进入项目目录
cd burncloud-download-aria2

# 运行单个测试（需要添加 --ignored 标志）
cargo test --test daemon_integration_test test_download_baidu_favicon -- --ignored --nocapture

# 或者运行所有集成测试
cargo test --test daemon_integration_test -- --ignored --nocapture
```

**参数说明**:
- `--ignored`: 运行标记为 ignore 的测试（需要网络和 aria2 二进制）
- `--nocapture`: 显示测试中的 println! 输出

## 预期输出

成功执行时应该看到类似输出：

```
📥 Starting download: https://www.baidu.com/favicon.ico
📁 Download directory: "C:\Users\huang\Work\burncloud\burncloud-download-aria2\test_downloads"
💾 Target file: "C:\Users\huang\Work\burncloud\burncloud-download-aria2\test_downloads\baidu_favicon.ico"
Download task created with ID: TaskId(...)
Progress: 0 / 1406 bytes (0.0%), Speed: 0 bytes/s
Progress: 1406 / 1406 bytes (100.0%), Speed: 2812 bytes/s
✅ Download completed successfully!

=================================
✅ Download completed successfully!
=================================
📊 File size: 1406 bytes
📁 File location: "C:\Users\huang\Work\burncloud\burncloud-download-aria2\test_downloads\baidu_favicon.ico"
💡 You can find the downloaded file at the path above
=================================

✅ File format validated as ICO
test test_download_baidu_favicon ... ok
```

## 查看下载文件

测试完成后，你可以在以下位置找到下载的文件：

**Windows**:
```
C:\Users\huang\Work\burncloud\burncloud-download-aria2\test_downloads\baidu_favicon.ico
```

**相对路径**:
```
burncloud-download-aria2/test_downloads/baidu_favicon.ico
```

你可以使用任何图像查看器打开这个 .ico 文件，或者在命令行中查看：

```bash
# Windows
explorer burncloud-download-aria2\test_downloads

# 查看文件信息
dir burncloud-download-aria2\test_downloads\baidu_favicon.ico
```

## 测试验证内容

1. ✅ **Daemon 自动启动**: 测试开始时自动启动 aria2 daemon
2. ✅ **任务创建**: 成功创建下载任务
3. ✅ **进度监控**: 能够获取并显示下载进度
4. ✅ **状态跟踪**: 正确跟踪任务状态变化
5. ✅ **文件完整性**: 验证文件存在且大小正确
6. ✅ **格式验证**: 验证下载的是有效的 ICO 文件（前4字节为 `00 00 01 00`）

## 测试特点

### 实时进度显示
测试会每 500ms 轮询一次进度，并在下载字节数变化时打印：
- 已下载字节数 / 总字节数
- 下载百分比
- 当前下载速度

### 超时保护
测试设置了 30 秒超时，防止网络问题导致测试挂起。

### 文件保存位置 🎯
**重要**: 文件保存在 `test_downloads/` 目录，**不会自动删除**。这样你可以：
- 验证文件确实被下载了
- 检查文件内容和格式
- 手动打开和查看文件

如果需要清理，可以手动删除该目录：
```bash
# Windows
rmdir /s /q burncloud-download-aria2\test_downloads

# Linux/Mac
rm -rf burncloud-download-aria2/test_downloads
```

### 格式验证
验证下载文件的文件头，确保是有效的 ICO 格式：
```rust
// ICO 文件头: 0x00 0x00 0x01 0x00
assert_eq!(&file_content[0..4], &[0x00, 0x00, 0x01, 0x00]);
```

## 故障排除

### 测试失败: "Connection timeout"
- **原因**: 无法连接到 www.baidu.com
- **解决**: 检查网络连接，确保可以访问百度

### 测试失败: "Failed to create download manager"
- **原因**: aria2 daemon 启动失败
- **解决**:
  - 检查端口 6800 是否被占用
  - 查看 aria2 二进制是否成功下载到 `%LOCALAPPDATA%\BurnCloud\aria2c.exe`

### 测试失败: "Download timeout after 30 seconds"
- **原因**: 下载速度太慢或网络中断
- **解决**: 重试测试，或检查网络连接质量

### 测试失败: "File should be a valid ICO format"
- **原因**: 下载的文件不是预期的 ICO 格式
- **解决**: 检查是否被代理或防火墙拦截，导致下载了错误内容

## 扩展测试

你可以基于这个测试模板创建更多下载测试：

```rust
#[tokio::test]
#[ignore]
async fn test_download_custom_file() {
    let url = "https://example.com/your-file.zip".to_string();
    let target_path = temp_dir.path().join("your-file.zip");

    // ... 其余代码相同 ...
}
```

## 技术细节

### 使用的 API
- `Aria2DownloadManager::new()` - 创建管理器（自动启动 daemon）
- `manager.add_download()` - 添加下载任务
- `manager.get_task()` - 获取任务状态
- `manager.get_progress()` - 获取下载进度

### 状态流转
```
Waiting → Downloading → Completed
                     ↓
                   Failed
```

### 并发安全
测试使用完整的异步实现，所有操作都是线程安全的。

## 参考资料

- [aria2 文档](https://aria2.github.io/)
- [burncloud-download API](../burncloud-download/src/lib.rs)
- [集成测试文档](./TESTING.md)
