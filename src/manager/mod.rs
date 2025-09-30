pub mod mapper;
pub mod state;

use crate::client::{Aria2Client, types::Aria2Options};
use crate::error::Aria2Error;
use crate::poller::ProgressPoller;
use burncloud_download::{DownloadManager, TaskId, DownloadTask, DownloadProgress};
use async_trait::async_trait;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

pub struct Aria2DownloadManager {
    client: Arc<Aria2Client>,
    state: Arc<state::StateManager>,
    _poller: Arc<ProgressPoller>,
}

impl Aria2DownloadManager {
    pub fn new(rpc_url: String, secret: Option<String>) -> Self {
        let client = Arc::new(Aria2Client::new(rpc_url, secret));
        let state = Arc::new(state::StateManager::new());
        let poller = Arc::new(ProgressPoller::new(client.clone(), state.clone()));

        // Start progress poller
        poller.start();

        Self {
            client,
            state,
            _poller: poller,
        }
    }

    async fn detect_download_type(&self, url: &str) -> Result<DownloadType> {
        if url.starts_with("magnet:") {
            Ok(DownloadType::Magnet)
        } else if url.ends_with(".torrent") {
            Ok(DownloadType::Torrent)
        } else if url.ends_with(".metalink") || url.ends_with(".meta4") {
            Ok(DownloadType::Metalink)
        } else if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("ftp://") {
            Ok(DownloadType::Http)
        } else {
            Err(Aria2Error::InvalidUrl(format!("Unsupported URL scheme: {}", url)).into())
        }
    }
}

enum DownloadType {
    Http,
    Torrent,
    Metalink,
    Magnet,
}

#[async_trait]
impl DownloadManager for Aria2DownloadManager {
    async fn add_download(&self, url: String, target_path: PathBuf) -> Result<TaskId> {
        // Create task
        let task = DownloadTask::new(url.clone(), target_path.clone());
        let task_id = task.id;

        // Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let download_type = self.detect_download_type(&url).await?;

        // Extract directory and filename
        let dir = target_path.parent()
            .ok_or_else(|| Aria2Error::InvalidPath("Invalid target path".to_string()))?
            .to_string_lossy()
            .to_string();

        let filename = target_path.file_name()
            .map(|n| n.to_string_lossy().to_string());

        let options = Aria2Options {
            dir,
            out: filename,
        };

        // Add download to aria2
        let gid = match download_type {
            DownloadType::Http | DownloadType::Magnet => {
                self.client.add_uri(vec![url.clone()], options).await?
            }
            DownloadType::Torrent => {
                let torrent_data = reqwest::get(&url).await?.bytes().await?.to_vec();
                self.client.add_torrent(torrent_data, options).await?
            }
            DownloadType::Metalink => {
                let metalink_data = reqwest::get(&url).await?.bytes().await?.to_vec();
                self.client.add_metalink(metalink_data, options).await?
            }
        };

        // Store task state
        self.state.add_task(task, gid).await;

        Ok(task_id)
    }

    async fn pause_download(&self, task_id: TaskId) -> Result<()> {
        let gid = self.state.get_gid(task_id).await
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        self.client.pause(&gid).await?;
        Ok(())
    }

    async fn resume_download(&self, task_id: TaskId) -> Result<()> {
        let gid = self.state.get_gid(task_id).await
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        self.client.unpause(&gid).await?;
        Ok(())
    }

    async fn cancel_download(&self, task_id: TaskId) -> Result<()> {
        let gid = self.state.get_gid(task_id).await
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        self.client.remove(&gid).await?;
        self.state.remove_task(task_id).await;
        Ok(())
    }

    async fn get_progress(&self, task_id: TaskId) -> Result<DownloadProgress> {
        let gid = self.state.get_gid(task_id).await
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        let status = self.client.tell_status(&gid).await?;

        let total_bytes = status.total_length.parse::<u64>().unwrap_or(0);
        let downloaded_bytes = status.completed_length.parse::<u64>().unwrap_or(0);
        let speed_bps = status.download_speed.parse::<u64>().unwrap_or(0);

        let eta_seconds = if speed_bps > 0 && total_bytes > downloaded_bytes {
            Some((total_bytes - downloaded_bytes) / speed_bps)
        } else {
            None
        };

        Ok(DownloadProgress {
            downloaded_bytes,
            total_bytes: if total_bytes > 0 { Some(total_bytes) } else { None },
            speed_bps,
            eta_seconds,
        })
    }

    async fn get_task(&self, task_id: TaskId) -> Result<DownloadTask> {
        let gid = self.state.get_gid(task_id).await
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        let state = self.state.get_state(&gid).await
            .ok_or_else(|| anyhow::anyhow!("Task state not found"))?;

        // Fetch latest status from aria2
        let aria2_status = self.client.tell_status(&gid).await?;
        let mut task = state.task.clone();
        task.status = mapper::map_aria2_status(&aria2_status);
        task.updated_at = std::time::SystemTime::now();

        // Update cached state
        self.state.update_task(&gid, task.clone()).await;

        Ok(task)
    }

    async fn list_tasks(&self) -> Result<Vec<DownloadTask>> {
        let states = self.state.list_all_states().await;
        let mut tasks = Vec::new();

        for state in states {
            // Fetch latest status for each task
            if let Ok(aria2_status) = self.client.tell_status(&state.aria2_gid).await {
                let mut task = state.task.clone();
                task.status = mapper::map_aria2_status(&aria2_status);
                task.updated_at = std::time::SystemTime::now();
                tasks.push(task);
            } else {
                tasks.push(state.task);
            }
        }

        Ok(tasks)
    }

    async fn active_download_count(&self) -> Result<usize> {
        let tasks = self.list_tasks().await?;
        Ok(tasks.iter().filter(|t| t.status.is_active()).count())
    }
}