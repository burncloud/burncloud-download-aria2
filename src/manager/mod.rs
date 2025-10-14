// Removed: pub mod mapper;
// Removed: pub mod state;

use crate::client::{Aria2Client, types::Aria2Options};
use crate::error::Aria2Error;
use crate::poller::ProgressPoller;
use burncloud_download_types::{TaskId, DownloadTask, DownloadProgress, DownloadManager};
use async_trait::async_trait;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use std::collections::HashMap;

pub struct Aria2DownloadManager {
    client: Arc<Aria2Client>,
    _poller: Arc<ProgressPoller>,
    _daemon: Arc<crate::daemon::Aria2Daemon>,
    // Map TaskId to GID for task identification
    task_gid_map: Arc<tokio::sync::RwLock<HashMap<TaskId, String>>>,
}

impl Aria2DownloadManager {
    pub async fn new(rpc_url: String, secret: Option<String>) -> Result<Self> {
        // 1. Create client first
        let client = Arc::new(Aria2Client::new(rpc_url.clone(), secret.clone()));

        // 2. Extract port from RPC URL (default to 6800 if not found)
        let rpc_port = Self::extract_port_from_url(&rpc_url).unwrap_or(6800);

        // 3. Start daemon with client for health checks
        let daemon_config = crate::daemon::DaemonConfig {
            rpc_port,
            rpc_secret: secret.unwrap_or_else(|| "burncloud".to_string()),
            ..Default::default()
        };
        let daemon = Arc::new(crate::daemon::Aria2Daemon::start(daemon_config, client.clone()).await?);

        // 4. Initialize poller without state manager
        let poller = Arc::new(ProgressPoller::new(client.clone()));

        // Start progress poller
        poller.start();

        Ok(Self {
            client,
            _poller: poller,
            _daemon: daemon,
            task_gid_map: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }

    /// Extract port number from RPC URL
    /// Examples: "http://localhost:6800/jsonrpc" -> Some(6800)
    fn extract_port_from_url(url: &str) -> Option<u16> {
        url.split(':')
            .nth(2)?
            .split('/')
            .next()?
            .parse()
            .ok()
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

    /// Get all tasks from aria2 RPC calls - returns raw JSON for real-time data
    async fn get_all_aria2_tasks(&self) -> Result<Vec<serde_json::Value>> {
        let mut all_tasks = Vec::new();

        // Get active downloads
        if let Ok(active) = self.client.tell_active().await {
            if let Some(active_array) = active.as_array() {
                all_tasks.extend(active_array.clone());
            }
        }

        // Get waiting downloads (limit to 1000)
        if let Ok(waiting) = self.client.tell_waiting(0, 1000).await {
            if let Some(waiting_array) = waiting.as_array() {
                all_tasks.extend(waiting_array.clone());
            }
        }

        // Get stopped downloads (limit to 1000)
        if let Ok(stopped) = self.client.tell_stopped(0, 1000).await {
            if let Some(stopped_array) = stopped.as_array() {
                all_tasks.extend(stopped_array.clone());
            }
        }

        Ok(all_tasks)
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
        // Check for duplicate URL first
        let existing_tasks = self.get_all_aria2_tasks().await?;
        let task_map = self.task_gid_map.read().await;

        for aria2_task in &existing_tasks {
            // Extract GID from the task
            let gid = aria2_task
                .get("gid")
                .and_then(|g| g.as_str())
                .unwrap_or("");

            // Extract URLs from files[].uris[]
            if let Some(files) = aria2_task.get("files").and_then(|f| f.as_array()) {
                for file in files {
                    if let Some(uris) = file.get("uris").and_then(|u| u.as_array()) {
                        for uri_obj in uris {
                            if let Some(existing_url) = uri_obj.get("uri").and_then(|u| u.as_str()) {
                                if existing_url == url {
                                    // Found duplicate URL, return existing TaskId if we have it
                                    if let Some(&existing_task_id) = task_map.iter()
                                        .find(|(_, task_gid)| task_gid.as_str() == gid)
                                        .map(|(task_id, _)| task_id) {
                                        return Ok(existing_task_id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        drop(task_map); // Release the read lock

        // No duplicate found, create new task
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

        // Store TaskId to GID mapping for later retrieval
        {
            let mut map = self.task_gid_map.write().await;
            map.insert(task_id, gid);
        }

        Ok(task_id)
    }

    async fn pause_download(&self, task_id: TaskId) -> Result<()> {
        let gid = {
            let map = self.task_gid_map.read().await;
            map.get(&task_id).cloned()
                .ok_or_else(|| anyhow::anyhow!("Task not found"))?
        };

        self.client.pause(&gid).await?;
        Ok(())
    }

    async fn resume_download(&self, task_id: TaskId) -> Result<()> {
        let gid = {
            let map = self.task_gid_map.read().await;
            map.get(&task_id).cloned()
                .ok_or_else(|| anyhow::anyhow!("Task not found"))?
        };

        self.client.unpause(&gid).await?;
        Ok(())
    }

    async fn cancel_download(&self, task_id: TaskId) -> Result<()> {
        let gid = {
            let mut map = self.task_gid_map.write().await;
            map.remove(&task_id)
                .ok_or_else(|| anyhow::anyhow!("Task not found"))?
        };

        self.client.remove(&gid).await?;
        Ok(())
    }

    async fn get_progress(&self, task_id: TaskId) -> Result<DownloadProgress> {
        let gid = {
            let map = self.task_gid_map.read().await;
            map.get(&task_id).cloned()
                .ok_or_else(|| anyhow::anyhow!("Task not found"))?
        };

        // Get real-time status from aria2
        let status = self.client.tell_status(&gid).await?;

        // Extract values directly from JSON for real-time data
        let total_bytes = status
            .get("totalLength")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let downloaded_bytes = status
            .get("completedLength")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let speed_bps = status
            .get("downloadSpeed")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

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
        let gid = {
            let map = self.task_gid_map.read().await;
            map.get(&task_id).cloned()
                .ok_or_else(|| anyhow::anyhow!("Task not found"))?
        };

        // Fetch latest status from aria2 RPC for real-time data
        let aria2_status = self.client.tell_status(&gid).await?;

        // Reconstruct basic task info from aria2 status JSON
        // Extract target path from files array
        let target_path = aria2_status
            .get("files")
            .and_then(|f| f.as_array())
            .and_then(|arr| arr.first())
            .and_then(|file| file.get("path"))
            .and_then(|path| path.as_str())
            .map(|path| PathBuf::from(path))
            .unwrap_or_else(|| PathBuf::from("unknown"));

        let mut task = DownloadTask::new("".to_string(), target_path);
        task.id = task_id;
        // Removed: task.status = ... - no status mapping, use real-time aria2 data
        task.updated_at = std::time::SystemTime::now();

        Ok(task)
    }

    async fn list_tasks(&self) -> Result<Vec<DownloadTask>> {
        // Get all aria2 tasks directly from RPC for real-time data
        let aria2_tasks = self.get_all_aria2_tasks().await?;
        let mut tasks = Vec::new();

        let task_map = self.task_gid_map.read().await;

        for aria2_status in aria2_tasks {
            // Extract GID from JSON
            let gid = aria2_status
                .get("gid")
                .and_then(|g| g.as_str())
                .unwrap_or("");

            // Try to find the TaskId for this GID
            if let Some(&task_id) = task_map.iter()
                .find(|(_, task_gid)| task_gid.as_str() == gid)
                .map(|(task_id, _)| task_id) {

                // Reconstruct task info from JSON
                let target_path = aria2_status
                    .get("files")
                    .and_then(|f| f.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|file| file.get("path"))
                    .and_then(|path| path.as_str())
                    .map(|path| PathBuf::from(path))
                    .unwrap_or_else(|| PathBuf::from("unknown"));

                let mut task = DownloadTask::new("".to_string(), target_path);
                task.id = task_id;
                // Removed: task.status = ... - no status mapping, use real-time aria2 data
                task.updated_at = std::time::SystemTime::now();

                tasks.push(task);
            }
        }

        Ok(tasks)
    }

    async fn active_download_count(&self) -> Result<usize> {
        // Get active downloads directly from aria2 for real-time count
        let active = self.client.tell_active().await?;

        // Count items in the JSON array
        let count = active
            .as_array()
            .map(|arr| arr.len())
            .unwrap_or(0);

        Ok(count)
    }
}