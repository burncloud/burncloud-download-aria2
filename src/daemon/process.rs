use crate::error::Aria2Error;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::process::{Command, Child};
use std::process::Stdio;

/// Configuration for the aria2 process
#[derive(Clone)]
pub struct ProcessConfig {
    pub rpc_port: u16,
    pub rpc_secret: String,
    pub download_dir: PathBuf,
    pub session_file: PathBuf,
    pub max_restart_attempts: u32,
}

/// Handle to manage the aria2 process lifecycle
pub struct ProcessHandle {
    child: Arc<Mutex<Option<Child>>>,
    restart_count: Arc<Mutex<u32>>,
    binary_path: PathBuf,
    config: ProcessConfig,
}

impl ProcessHandle {
    pub fn new(binary_path: PathBuf, config: ProcessConfig) -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            restart_count: Arc::new(Mutex::new(0)),
            binary_path,
            config,
        }
    }

    /// Start the aria2 process
    pub async fn start_process(&self) -> Result<(), Aria2Error> {
        // Stop existing process if running
        self.stop_process().await?;

        // Ensure download directory exists and is accessible
        tokio::fs::create_dir_all(&self.config.download_dir).await
            .map_err(|e| Aria2Error::ProcessStartFailed(
                format!("Failed to create download directory {:?}: {}", self.config.download_dir, e)
            ))?;

        // Ensure session file parent directory exists
        if let Some(parent) = self.config.session_file.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| Aria2Error::ProcessStartFailed(
                    format!("Failed to create session directory {:?}: {}", parent, e)
                ))?;
        }

        // Build command
        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("--enable-rpc")
            .arg("--rpc-listen-port")
            .arg(self.config.rpc_port.to_string())
            .arg("--rpc-secret")
            .arg(&self.config.rpc_secret)
            .arg("--dir")
            .arg(self.config.download_dir.to_string_lossy().as_ref())
            .arg("--continue")
            .arg("--save-session")
            .arg(self.config.session_file.to_string_lossy().as_ref())
            .arg("--save-session-interval")
            .arg("60");

        // Only add --input-file if session file exists
        if self.config.session_file.exists() {
            cmd.arg("--input-file")
                .arg(self.config.session_file.to_string_lossy().as_ref());
        }

        // Don't suppress output in debug builds to help with troubleshooting
        #[cfg(debug_assertions)]
        {
            cmd.stdout(Stdio::piped())
                .stderr(Stdio::piped());
        }

        #[cfg(not(debug_assertions))]
        {
            cmd.arg("--quiet")
                .stdout(Stdio::null())
                .stderr(Stdio::null());
        }

        cmd.kill_on_drop(true);

        // In debug builds, log the command that will be executed
        #[cfg(debug_assertions)]
        {
            eprintln!("DEBUG: Starting aria2c with command:");
            eprintln!("  Binary: {:?}", self.binary_path);
            eprintln!("  Args: {:?}", cmd);
        }

        // Spawn process
        let mut child = cmd.spawn()
            .map_err(|e| Aria2Error::ProcessStartFailed(format!("Failed to spawn aria2: {}", e)))?;

        // In debug builds, capture stderr for diagnostics
        #[cfg(debug_assertions)]
        {
            if let Some(stderr) = child.stderr.take() {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let stderr_reader = BufReader::new(stderr);
                tokio::spawn(async move {
                    let mut lines = stderr_reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        eprintln!("aria2 stderr: {}", line);
                    }
                });
            }
        }

        // Store child process
        *self.child.lock().await = Some(child);

        Ok(())
    }

    /// Stop the aria2 process
    pub async fn stop_process(&self) -> Result<(), Aria2Error> {
        let mut child_guard = self.child.lock().await;

        if let Some(mut child) = child_guard.take() {
            // Try to kill gracefully
            if let Err(e) = child.kill().await {
                // Process might have already exited
                if e.kind() != std::io::ErrorKind::InvalidInput {
                    return Err(Aria2Error::ProcessManagementError(
                        format!("Failed to kill process: {}", e)
                    ));
                }
            }

            // Wait for process to exit
            let _ = child.wait().await;
        }

        Ok(())
    }

    /// Check if the process is running
    pub async fn is_running(&self) -> bool {
        let mut child_guard = self.child.lock().await;

        if let Some(child) = child_guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    *child_guard = None;
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(_) => {
                    // Error checking status, assume not running
                    *child_guard = None;
                    false
                }
            }
        } else {
            false
        }
    }

    /// Increment and return the restart counter
    pub async fn increment_restart_count(&self) -> u32 {
        let mut count = self.restart_count.lock().await;
        *count += 1;
        *count
    }

    /// Reset the restart counter
    pub async fn reset_restart_count(&self) {
        *self.restart_count.lock().await = 0;
    }

    /// Get the max restart attempts from config
    pub fn max_restart_attempts(&self) -> u32 {
        self.config.max_restart_attempts
    }
}
