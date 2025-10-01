use crate::error::Aria2Error;
use crate::client::Aria2Client;
use super::process::ProcessHandle;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

/// Health monitor for aria2 process
pub struct HealthMonitor {
    process: Arc<ProcessHandle>,
    client: Arc<Aria2Client>,
    shutdown: Arc<Notify>,
    check_interval: Duration,
}

impl HealthMonitor {
    pub fn new(
        process: Arc<ProcessHandle>,
        client: Arc<Aria2Client>,
        check_interval: Duration,
    ) -> Self {
        Self {
            process,
            client,
            shutdown: Arc::new(Notify::new()),
            check_interval,
        }
    }

    /// Start the health monitoring loop in a background task
    pub fn start(&self) {
        let process = self.process.clone();
        let client = self.client.clone();
        let shutdown = self.shutdown.clone();
        let check_interval = self.check_interval;

        tokio::spawn(async move {
            Self::monitor_loop(process, client, shutdown, check_interval).await;
        });
    }

    /// Main monitoring loop
    async fn monitor_loop(
        process: Arc<ProcessHandle>,
        client: Arc<Aria2Client>,
        shutdown: Arc<Notify>,
        check_interval: Duration,
    ) {
        let mut interval = tokio::time::interval(check_interval);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if !process.is_running().await {
                        // Process crashed, try to restart
                        #[allow(clippy::redundant_pattern_matching)]
                        if let Err(_) = Self::handle_crash(&process).await {
                            // Restart limit exceeded, exit monitor
                            break;
                        }
                    } else if Self::check_health(&client).await {
                        // Process is healthy, reset restart counter
                        process.reset_restart_count().await;
                    }
                    // If not healthy but process running, it might be starting up
                }
                _ = shutdown.notified() => {
                    // Shutdown requested
                    break;
                }
            }
        }
    }

    /// Check if aria2 RPC is responding
    async fn check_health(client: &Arc<Aria2Client>) -> bool {
        client.get_global_stat().await.is_ok()
    }

    /// Handle process crash with restart logic
    async fn handle_crash(process: &Arc<ProcessHandle>) -> Result<(), Aria2Error> {
        let restart_count = process.increment_restart_count().await;

        if restart_count > process.max_restart_attempts() {
            return Err(Aria2Error::RestartLimitExceeded);
        }

        // Exponential backoff: 2^n seconds, max 60s
        let backoff_secs = std::cmp::min(1u64 << (restart_count - 1), 60);
        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;

        // Try to restart
        process.start_process().await
    }

    /// Request shutdown of the monitor
    pub fn shutdown(&self) {
        self.shutdown.notify_one();
    }
}
