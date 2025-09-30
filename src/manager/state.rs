use burncloud_download::{TaskId, DownloadTask};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Internal task state mapping
#[derive(Debug, Clone)]
pub struct TaskState {
    pub task: DownloadTask,
    pub aria2_gid: String,
}

/// Thread-safe state manager
pub struct StateManager {
    task_to_gid: RwLock<HashMap<TaskId, String>>,
    gid_to_state: RwLock<HashMap<String, TaskState>>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            task_to_gid: RwLock::new(HashMap::new()),
            gid_to_state: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_task(&self, task: DownloadTask, gid: String) {
        let task_id = task.id;
        let state = TaskState {
            task: task.clone(),
            aria2_gid: gid.clone(),
        };

        let mut task_to_gid = self.task_to_gid.write().await;
        let mut gid_to_state = self.gid_to_state.write().await;

        task_to_gid.insert(task_id, gid.clone());
        gid_to_state.insert(gid, state);
    }

    pub async fn get_gid(&self, task_id: TaskId) -> Option<String> {
        let task_to_gid = self.task_to_gid.read().await;
        task_to_gid.get(&task_id).cloned()
    }

    pub async fn get_state(&self, gid: &str) -> Option<TaskState> {
        let gid_to_state = self.gid_to_state.read().await;
        gid_to_state.get(gid).cloned()
    }

    pub async fn update_task(&self, gid: &str, task: DownloadTask) {
        let mut gid_to_state = self.gid_to_state.write().await;
        if let Some(state) = gid_to_state.get_mut(gid) {
            state.task = task;
        }
    }

    pub async fn remove_task(&self, task_id: TaskId) -> Option<String> {
        let mut task_to_gid = self.task_to_gid.write().await;
        let gid = task_to_gid.remove(&task_id)?;

        let mut gid_to_state = self.gid_to_state.write().await;
        gid_to_state.remove(&gid);

        Some(gid)
    }

    pub async fn list_all_states(&self) -> Vec<TaskState> {
        let gid_to_state = self.gid_to_state.read().await;
        gid_to_state.values().cloned().collect()
    }
}