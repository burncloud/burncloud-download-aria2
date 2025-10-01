use crate::error::Aria2Error;
use crate::client::Aria2Client;
use super::{platform, binary, process, monitor};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Configuration for the Aria2 daemon
#[derive(Clone)]
pub struct DaemonConfig {
    pub rpc_port: u16,
    pub rpc_secret: String,
    pub download_dir: std::path::PathBuf,
    pub max_restart_attempts: u32,
    pub health_check_interval: Duration,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            rpc_port: 6800,
            rpc_secret: "burncloud".to_string(),
            download_dir: platform::get_binary_dir(),
            max_restart_attempts: 10,
            health_check_interval: Duration::from_secs(10),
        }
    }
}

/// Main Aria2 daemon orchestrator
pub struct Aria2Daemon {
    process: Arc<process::ProcessHandle>,
    monitor: Arc<monitor::HealthMonitor>,
}

impl Aria2Daemon {
    /// Start the aria2 daemon
    pub async fn start(config: DaemonConfig, client: Arc<Aria2Client>) -> Result<Self, Aria2Error> {
        // 1. Get binary path
        let binary_path = platform::get_binary_path();

        // 2. Download binary if missing
        if !binary::verify_binary_exists(&binary_path).await {
            binary::download_aria2_binary(&binary_path).await?;
        }

        // 3. Ensure download directory exists
        platform::ensure_directory(&config.download_dir).await?;

        // 4. Create process handle with config
        let process_config = process::ProcessConfig {
            rpc_port: config.rpc_port,
            rpc_secret: config.rpc_secret.clone(),
            download_dir: config.download_dir.clone(),
            max_restart_attempts: config.max_restart_attempts,
        };
        let process = Arc::new(process::ProcessHandle::new(binary_path, process_config));

        // 5. Start process
        process.start_process().await?;

        // 6. Wait for RPC to be ready (max 30 seconds)
        let start_time = Instant::now();
        let timeout = Duration::from_secs(30);

        while start_time.elapsed() < timeout {
            if client.get_global_stat().await.is_ok() {
                break;
            }

            if start_time.elapsed() >= timeout {
                return Err(Aria2Error::DaemonUnavailable(
                    "RPC not ready after 30 seconds".to_string()
                ));
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // 7. Start health monitor
        let monitor = Arc::new(monitor::HealthMonitor::new(
            process.clone(),
            client,
            config.health_check_interval,
        ));
        monitor.start();

        Ok(Self { process, monitor })
    }

    /// Stop the daemon
    pub async fn stop(&self) -> Result<(), Aria2Error> {
        // Stop monitoring
        self.monitor.shutdown();

        // Stop process
        self.process.stop_process().await
    }

    /// Check if daemon is healthy
    pub async fn is_healthy(&self) -> bool {
        self.process.is_running().await
    }
}

impl Drop for Aria2Daemon {
    fn drop(&mut self) {
        // Stop monitor
        self.monitor.shutdown();

        // Stop process (using block_in_place for Drop)
        let process = self.process.clone();
        let _ = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(process.stop_process())
        });
    }
}
