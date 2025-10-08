pub mod aggregator;

use crate::client::Aria2Client;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub struct ProgressPoller {
    client: Arc<Aria2Client>,
    shutdown: Arc<tokio::sync::Notify>,
}

impl ProgressPoller {
    pub fn new(client: Arc<Aria2Client>) -> Self {
        Self {
            client,
            shutdown: Arc::new(tokio::sync::Notify::new()),
        }
    }

    pub fn start(&self) {
        let _client = self.client.clone();
        let shutdown = self.shutdown.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        // Progress polling is now handled by real-time RPC calls
                        // This poller can be used for other background tasks if needed
                        // For now, it just maintains the interval structure
                    }
                    _ = shutdown.notified() => {
                        break;
                    }
                }
            }
        });
    }

    pub fn shutdown(&self) {
        self.shutdown.notify_one();
    }
}

impl Drop for ProgressPoller {
    fn drop(&mut self) {
        self.shutdown.notify_one();
    }
}