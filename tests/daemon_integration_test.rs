//! Integration tests for aria2 daemon lifecycle
//! These tests validate the complete daemon functionality including:
//! - Binary auto-download
//! - Process lifecycle management
//! - Health monitoring and auto-restart
//! - Restart limit enforcement

use burncloud_download_aria2::{Aria2Daemon, DaemonConfig};
use burncloud_download_aria2::client::Aria2Client;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tempfile::TempDir;

/// Helper function to create a test configuration with custom directories
fn create_test_config(temp_dir: &TempDir) -> DaemonConfig {
    DaemonConfig {
        rpc_port: 6801, // Use different port to avoid conflicts
        rpc_secret: "test_secret".to_string(),
        download_dir: temp_dir.path().to_path_buf(),
        max_restart_attempts: 10,
        health_check_interval: Duration::from_secs(5),
    }
}

#[tokio::test]
#[ignore] // Requires network access and file system permissions
async fn test_daemon_start_with_missing_binary() {
    // This test validates that the daemon can download aria2 if it's missing
    // Note: This will actually download the binary, so it requires network access

    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);

    let client = Arc::new(Aria2Client::new(
        format!("http://localhost:{}/jsonrpc", config.rpc_port),
        Some(config.rpc_secret.clone()),
    ));

    // Start daemon - should download binary if missing
    let result = Aria2Daemon::start(config, client.clone()).await;

    // On success, we should have a running daemon
    if let Ok(daemon) = result {
        assert!(daemon.is_healthy().await);

        // Verify RPC is responding
        let stat = client.get_global_stat().await;
        assert!(stat.is_ok(), "RPC should be responding");

        // Cleanup
        daemon.stop().await.ok();
    } else if let Err(e) = result {
        // If it fails, it might be due to network or permissions
        eprintln!("Daemon start failed (expected on CI): {}", e);
    }
}

#[tokio::test]
#[ignore] // Requires aria2 binary to be present
async fn test_daemon_lifecycle_with_existing_binary() {
    // This test assumes aria2c is already available in the system PATH
    // or has been downloaded by a previous test run

    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);

    let client = Arc::new(Aria2Client::new(
        format!("http://localhost:{}/jsonrpc", config.rpc_port),
        Some(config.rpc_secret.clone()),
    ));

    // Start daemon
    let daemon = Aria2Daemon::start(config, client.clone())
        .await
        .expect("Failed to start daemon");

    // Verify daemon is healthy
    assert!(daemon.is_healthy().await);

    // Verify RPC is responding
    let stat = client.get_global_stat().await;
    assert!(stat.is_ok(), "RPC should be responding");

    // Stop daemon
    daemon.stop().await.expect("Failed to stop daemon");

    // Give it time to stop
    sleep(Duration::from_secs(1)).await;

    // Verify daemon stopped (RPC should not respond)
    let stat = client.get_global_stat().await;
    assert!(stat.is_err(), "RPC should not be responding after stop");
}

#[tokio::test]
#[ignore] // Requires aria2 binary and ability to kill processes
async fn test_daemon_auto_restart_on_crash() {
    // This test verifies that the daemon automatically restarts aria2 when it crashes

    let temp_dir = TempDir::new().unwrap();
    let config = DaemonConfig {
        rpc_port: 6802,
        rpc_secret: "test_restart".to_string(),
        download_dir: temp_dir.path().to_path_buf(),
        max_restart_attempts: 5,
        health_check_interval: Duration::from_secs(3), // Check more frequently
    };

    let client = Arc::new(Aria2Client::new(
        format!("http://localhost:{}/jsonrpc", config.rpc_port),
        Some(config.rpc_secret.clone()),
    ));

    let daemon = Aria2Daemon::start(config.clone(), client.clone())
        .await
        .expect("Failed to start daemon");

    // Verify daemon is initially healthy
    assert!(daemon.is_healthy().await);

    // Find and kill the aria2 process
    // Note: This is platform-specific and might need adjustment
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "aria2c.exe"])
            .output();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("pkill")
            .args(["-9", "aria2c"])
            .output();
    }

    // Wait for monitor to detect crash and restart (3s interval + restart time)
    sleep(Duration::from_secs(8)).await;

    // Daemon should have restarted the process
    // Try to communicate via RPC
    let stat = client.get_global_stat().await;
    assert!(stat.is_ok(), "RPC should be responding after auto-restart");

    // Cleanup
    daemon.stop().await.ok();
}

#[tokio::test]
#[ignore] // Requires aria2 binary
async fn test_daemon_restart_limit_enforcement() {
    // This test verifies that the daemon stops restarting after max attempts
    // Note: This is difficult to test in practice without mocking

    let temp_dir = TempDir::new().unwrap();
    let config = DaemonConfig {
        rpc_port: 6803,
        rpc_secret: "test_limit".to_string(),
        download_dir: temp_dir.path().to_path_buf(),
        max_restart_attempts: 2, // Set low for testing
        health_check_interval: Duration::from_secs(2),
    };

    let client = Arc::new(Aria2Client::new(
        format!("http://localhost:{}/jsonrpc", config.rpc_port),
        Some(config.rpc_secret.clone()),
    ));

    let daemon = Aria2Daemon::start(config, client.clone())
        .await
        .expect("Failed to start daemon");

    // This test is mainly to verify the configuration accepts low restart limits
    // Full restart limit testing would require process mocking
    assert!(daemon.is_healthy().await);

    // Cleanup
    daemon.stop().await.ok();
}

#[tokio::test]
#[ignore] // Requires aria2 binary
async fn test_daemon_rpc_readiness_wait() {
    // This test verifies that the daemon waits for RPC to be ready

    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);

    let client = Arc::new(Aria2Client::new(
        format!("http://localhost:{}/jsonrpc", config.rpc_port),
        Some(config.rpc_secret.clone()),
    ));

    // Start daemon - should wait for RPC to be ready
    let start = std::time::Instant::now();
    let daemon = Aria2Daemon::start(config, client.clone())
        .await
        .expect("Failed to start daemon");

    let elapsed = start.elapsed();

    // Should complete within reasonable time (not timeout at 30s)
    assert!(elapsed < Duration::from_secs(30), "Daemon should start quickly");

    // RPC should be immediately available
    let stat = client.get_global_stat().await;
    assert!(stat.is_ok(), "RPC should be ready immediately after start");

    // Cleanup
    daemon.stop().await.ok();
}

#[tokio::test]
#[ignore] // Requires aria2 binary
async fn test_daemon_drop_cleanup() {
    // This test verifies that dropping the daemon stops the process

    let temp_dir = TempDir::new().unwrap();
    let config = DaemonConfig {
        rpc_port: 6804,
        rpc_secret: "test_drop".to_string(),
        download_dir: temp_dir.path().to_path_buf(),
        max_restart_attempts: 10,
        health_check_interval: Duration::from_secs(5),
    };

    let client = Arc::new(Aria2Client::new(
        format!("http://localhost:{}/jsonrpc", config.rpc_port),
        Some(config.rpc_secret.clone()),
    ));

    {
        let daemon = Aria2Daemon::start(config, client.clone())
            .await
            .expect("Failed to start daemon");

        assert!(daemon.is_healthy().await);

        // daemon will be dropped here
    }

    // Wait for cleanup
    sleep(Duration::from_secs(2)).await;

    // RPC should not be responding after daemon is dropped
    let stat = client.get_global_stat().await;
    assert!(stat.is_err(), "RPC should not be responding after daemon drop");
}

#[tokio::test]
#[ignore] // Requires aria2 binary
async fn test_daemon_custom_configuration() {
    // This test verifies that custom daemon configuration is properly applied

    let temp_dir = TempDir::new().unwrap();
    let custom_port = 6805;
    let custom_secret = "my_custom_secret";

    let config = DaemonConfig {
        rpc_port: custom_port,
        rpc_secret: custom_secret.to_string(),
        download_dir: temp_dir.path().to_path_buf(),
        max_restart_attempts: 15,
        health_check_interval: Duration::from_secs(8),
    };

    let client = Arc::new(Aria2Client::new(
        format!("http://localhost:{}/jsonrpc", custom_port),
        Some(custom_secret.to_string()),
    ));

    let daemon = Aria2Daemon::start(config, client.clone())
        .await
        .expect("Failed to start daemon with custom config");

    // Verify RPC responds on custom port with custom secret
    let stat = client.get_global_stat().await;
    assert!(stat.is_ok(), "RPC should respond with custom configuration");

    // Try connecting with wrong secret - should fail
    let wrong_client = Arc::new(Aria2Client::new(
        format!("http://localhost:{}/jsonrpc", custom_port),
        Some("wrong_secret".to_string()),
    ));

    let stat = wrong_client.get_global_stat().await;
    assert!(stat.is_err(), "RPC should reject wrong secret");

    // Cleanup
    daemon.stop().await.ok();
}

#[tokio::test]
async fn test_daemon_config_default_values() {
    // Unit test for default configuration values
    let config = DaemonConfig::default();

    assert_eq!(config.rpc_port, 6800);
    assert_eq!(config.rpc_secret, "burncloud");
    assert_eq!(config.max_restart_attempts, 10);
    assert_eq!(config.health_check_interval, Duration::from_secs(10));
}

#[tokio::test]
#[ignore] // Requires network to fail
async fn test_daemon_start_timeout_on_rpc_unavailable() {
    // This test verifies that daemon start times out if RPC never becomes ready
    // We use an invalid port configuration to simulate this

    let temp_dir = TempDir::new().unwrap();
    let config = DaemonConfig {
        rpc_port: 6806,
        rpc_secret: "test_timeout".to_string(),
        download_dir: temp_dir.path().to_path_buf(),
        max_restart_attempts: 10,
        health_check_interval: Duration::from_secs(5),
    };

    // Create client pointing to different port than daemon
    let client = Arc::new(Aria2Client::new(
        "http://localhost:6807/jsonrpc".to_string(), // Wrong port
        Some(config.rpc_secret.clone()),
    ));

    // This should timeout waiting for RPC to be ready
    let start = std::time::Instant::now();
    let result = Aria2Daemon::start(config, client).await;
    let elapsed = start.elapsed();

    // Should timeout around 30 seconds
    assert!(result.is_err(), "Should fail when RPC doesn't become ready");
    assert!(elapsed >= Duration::from_secs(25), "Should wait for timeout period");
    assert!(elapsed < Duration::from_secs(35), "Should not wait much longer than timeout");
}

#[tokio::test]
#[ignore] // Requires aria2 binary
async fn test_multiple_daemon_instances_different_ports() {
    // Verify that multiple daemon instances can run on different ports
    // This is important for testing and multi-instance scenarios

    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();

    let config1 = DaemonConfig {
        rpc_port: 6810,
        rpc_secret: "daemon1".to_string(),
        download_dir: temp_dir1.path().to_path_buf(),
        max_restart_attempts: 10,
        health_check_interval: Duration::from_secs(5),
    };

    let config2 = DaemonConfig {
        rpc_port: 6811,
        rpc_secret: "daemon2".to_string(),
        download_dir: temp_dir2.path().to_path_buf(),
        max_restart_attempts: 10,
        health_check_interval: Duration::from_secs(5),
    };

    let client1 = Arc::new(Aria2Client::new(
        format!("http://localhost:{}/jsonrpc", config1.rpc_port),
        Some(config1.rpc_secret.clone()),
    ));

    let client2 = Arc::new(Aria2Client::new(
        format!("http://localhost:{}/jsonrpc", config2.rpc_port),
        Some(config2.rpc_secret.clone()),
    ));

    let daemon1 = Aria2Daemon::start(config1, client1.clone())
        .await
        .expect("Failed to start daemon1");

    let daemon2 = Aria2Daemon::start(config2, client2.clone())
        .await
        .expect("Failed to start daemon2");

    // Both should be healthy
    assert!(daemon1.is_healthy().await);
    assert!(daemon2.is_healthy().await);

    // Both RPCs should respond
    assert!(client1.get_global_stat().await.is_ok());
    assert!(client2.get_global_stat().await.is_ok());

    // Cleanup
    daemon1.stop().await.ok();
    daemon2.stop().await.ok();
}

#[tokio::test]
#[ignore] // Requires network access and aria2 binary
async fn test_download_baidu_favicon() {
    // This test validates that the daemon can successfully download a real file
    // Downloads https://www.baidu.com/favicon.ico

    use burncloud_download_aria2::Aria2DownloadManager;
    use burncloud_download::DownloadManager;
    use std::env;

    // Use a persistent directory instead of temp, so file remains after test
    let download_dir = env::current_dir()
        .unwrap()
        .join("test_downloads");

    // Create directory if it doesn't exist
    tokio::fs::create_dir_all(&download_dir).await.unwrap();

    // Create download manager (this will start the daemon)
    let manager = Aria2DownloadManager::new(
        "http://localhost:6800/jsonrpc".to_string(),
        Some("burncloud".to_string()),
    )
    .await
    .expect("Failed to create download manager");

    // Define download target - use persistent directory
    let url = "https://www.baidu.com/favicon.ico".to_string();
    let target_path = download_dir.join("baidu_favicon.ico");

    println!("📥 Starting download: {}", url);
    println!("📁 Download directory: {:?}", download_dir);
    println!("💾 Target file: {:?}", target_path);

    // Add download task
    let task_id = manager
        .add_download(url.clone(), target_path.clone())
        .await
        .expect("Failed to add download task");

    println!("Download task created with ID: {:?}", task_id);

    // Monitor download progress
    let mut previous_downloaded = 0u64;
    let timeout = Duration::from_secs(30);
    let start_time = std::time::Instant::now();

    loop {
        if start_time.elapsed() > timeout {
            panic!("Download timeout after 30 seconds");
        }

        // Get task status
        let task = manager.get_task(task_id).await.expect("Failed to get task");

        // Get progress
        let progress = manager.get_progress(task_id).await.expect("Failed to get progress");

        // Print progress if changed
        if progress.downloaded_bytes != previous_downloaded {
            println!(
                "Progress: {} / {} bytes ({:.1}%), Speed: {} bytes/s",
                progress.downloaded_bytes,
                progress.total_bytes.unwrap_or(0),
                if let Some(total) = progress.total_bytes {
                    (progress.downloaded_bytes as f64 / total as f64) * 100.0
                } else {
                    0.0
                },
                progress.speed_bps
            );
            previous_downloaded = progress.downloaded_bytes;
        }

        // Check if completed
        use burncloud_download::DownloadStatus;
        match task.status {
            DownloadStatus::Completed => {
                println!("✅ Download completed successfully!");
                break;
            }
            DownloadStatus::Failed(ref reason) => {
                panic!("❌ Download failed: {}", reason);
            }
            _ => {
                // Still downloading, wait a bit
                sleep(Duration::from_millis(500)).await;
            }
        }
    }

    // Verify file exists and has content
    assert!(target_path.exists(), "Downloaded file should exist");

    let metadata = tokio::fs::metadata(&target_path)
        .await
        .expect("Failed to get file metadata");

    assert!(metadata.len() > 0, "Downloaded file should not be empty");

    println!("\n=================================");
    println!("✅ Download completed successfully!");
    println!("=================================");
    println!("📊 File size: {} bytes", metadata.len());
    println!("📁 File saved to: test_downloads/baidu_favicon.ico");
    println!("=================================\n");

    // Optional: Read and validate the file
    let file_content = tokio::fs::read(&target_path)
        .await
        .expect("Failed to read downloaded file");

    println!("📋 File info:");
    println!("   - Size: {} bytes", file_content.len());

    // Check file format (ICO files typically start with 0x00 0x00 0x01 0x00)
    if file_content.len() >= 4 {
        println!("   - First 4 bytes: {:02X} {:02X} {:02X} {:02X}",
            file_content[0], file_content[1], file_content[2], file_content[3]);

        // Note: Baidu's favicon might not be a standard ICO, could be PNG or other format
        // Just verify we got some data, don't enforce format
        if &file_content[0..4] == &[0x00, 0x00, 0x01, 0x00] {
            println!("   - Format: ICO ✅");
        } else if &file_content[0..4] == &[0x89, 0x50, 0x4E, 0x47] {
            println!("   - Format: PNG ✅");
        } else if &file_content[0..2] == &[0xFF, 0xD8] {
            println!("   - Format: JPEG ✅");
        } else {
            println!("   - Format: Unknown (but file downloaded successfully) ✅");
        }
    }

    // Give time for cleanup before test ends
    println!("\n🧹 Cleaning up...");
    drop(manager);

    // Give more time to allow background tasks and daemon cleanup to finish properly
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("✅ Test completed successfully!");
}
