pub mod aggregator;

use crate::client::Aria2Client;
use crate::manager::{state::StateManager, mapper};
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub struct ProgressPoller {
    client: Arc<Aria2Client>,
    state: Arc<StateManager>,
    shutdown: Arc<tokio::sync::Notify>,
}

impl ProgressPoller {
    pub fn new(client: Arc<Aria2Client>, state: Arc<StateManager>) -> Self {
        Self {
            client,
            state,
            shutdown: Arc::new(tokio::sync::Notify::new()),
        }
    }

    pub fn start(&self) {
        let client = self.client.clone();
        let state = self.state.clone();
        let shutdown = self.shutdown.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        Self::poll_progress(&client, &state).await;
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

    async fn poll_progress(client: &Arc<Aria2Client>, state: &Arc<StateManager>) {
        let states = state.list_all_states().await;

        for task_state in states {
            if let Ok(aria2_status) = client.tell_status(&task_state.aria2_gid).await {
                let new_status = mapper::map_aria2_status(&aria2_status);

                if new_status != task_state.task.status {
                    let mut updated_task = task_state.task.clone();
                    updated_task.status = new_status;
                    updated_task.updated_at = std::time::SystemTime::now();
                    state.update_task(&task_state.aria2_gid, updated_task).await;
                }
            }
        }
    }
}

impl Drop for ProgressPoller {
    fn drop(&mut self) {
        self.shutdown.notify_one();
    }
}