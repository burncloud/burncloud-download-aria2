use burncloud_download_aria2::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 启动 BurnCloud Aria2 下载器");

    // 方式1: 使用 quick_start 快速启动
    println!("\n📦 初始化 aria2 管理器...");
    let mut manager = quick_start().await?;

    println!("✅ aria2 管理器启动成功！");

    // 创建 RPC 客户端
    if let Some(client) = manager.create_rpc_client() {
        println!("\n🔗 连接到 aria2 RPC 服务");

        // 获取全局统计信息
        match client.get_global_stat().await {
            Ok(stat) => {
                println!("📊 全局统计:");
                println!("  - 活跃下载: {}", stat.num_active);
                println!("  - 等待下载: {}", stat.num_waiting);
                println!("  - 下载速度: {} bytes/s", stat.download_speed);
            }
            Err(e) => println!("❌ 获取统计信息失败: {}", e),
        }

        // 添加一个测试下载
        println!("\n⬇️  添加测试下载任务...");
        let test_url = "https://httpbin.org/bytes/1024"; // 1KB 测试文件

        match client.add_uri(vec![test_url.to_string()], None).await {
            Ok(gid) => {
                println!("✅ 下载任务已添加, GID: {}", gid);

                // 监控下载状态
                for i in 0..10 {
                    match client.tell_status(&gid).await {
                        Ok(status) => {
                            println!("📈 下载状态 [{}]: {} - {}%",
                                i+1,
                                status.status,
                                if status.total_length.parse::<u64>().unwrap_or(0) > 0 {
                                    status.completed_length.parse::<u64>().unwrap_or(0) * 100
                                     / status.total_length.parse::<u64>().unwrap_or(1)
                                } else {
                                    0
                                }
                            );

                            if status.status == "complete" || status.status == "error" {
                                break;
                            }
                        }
                        Err(e) => println!("❌ 获取状态失败: {}", e),
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
            Err(e) => println!("❌ 添加下载失败: {}", e),
        }

        // 获取活跃下载列表
        println!("\n📋 获取活跃下载列表...");
        match client.tell_active().await {
            Ok(downloads) => {
                if downloads.is_empty() {
                    println!("📝 当前没有活跃下载");
                } else {
                    for download in downloads {
                        println!("📄 GID: {} - 状态: {} - 速度: {} bytes/s",
                            download.gid, download.status, download.download_speed);
                    }
                }
            }
            Err(e) => println!("❌ 获取活跃下载失败: {}", e),
        }
    } else {
        println!("❌ 无法创建 RPC 客户端");
    }

    println!("\n⏳ 运行5秒后自动退出...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // 关闭管理器
    println!("\n🛑 关闭 aria2 管理器...");
    manager.shutdown().await?;

    println!("✅ 程序执行完成！");
    Ok(())
}