//! # BurnCloud Aria2 Download Manager
//!
//! Aria2 backend implementation for BurnCloud download manager.
//!
//! ## Features
//!
//! - Custom JSON-RPC client for aria2 communication
//! - Support for HTTP/HTTPS, BitTorrent, Metalink, and Magnet downloads
//! - Progress polling every 1 second
//! - Multi-file download aggregate progress reporting
//! - Full DownloadManager trait implementation
//! - Automatic port conflict resolution - starts from port 6800 and increments if occupied

pub mod client;
pub mod manager;
pub mod poller;
pub mod error;
pub mod daemon;

pub use manager::Aria2DownloadManager;
pub use error::Aria2Error;
pub use daemon::{Aria2Daemon, DaemonConfig};

/// Default aria2 RPC endpoint
pub const DEFAULT_ARIA2_RPC_URL: &str = "http://localhost:6800/jsonrpc";

/// Default aria2 RPC secret token
pub const DEFAULT_ARIA2_SECRET: &str = "burncloud";

/// Create a new Aria2DownloadManager with automatic port conflict resolution.
/// If port 6800 is occupied, it will automatically find the next available port.
///
/// # Examples
///
/// ```no_run
/// use burncloud_download_aria2::create_manager_with_auto_port;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let manager = create_manager_with_auto_port().await?;
///     // Manager is now ready to use with an available port
///     Ok(())
/// }
/// ```
pub async fn create_manager_with_auto_port() -> anyhow::Result<Aria2DownloadManager> {
    Aria2DownloadManager::new(
        DEFAULT_ARIA2_RPC_URL.to_string(),
        Some(DEFAULT_ARIA2_SECRET.to_string())
    ).await
}

/// Create a new Aria2DownloadManager with custom secret and automatic port conflict resolution.
///
/// # Examples
///
/// ```no_run
/// use burncloud_download_aria2::create_manager_with_secret;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let manager = create_manager_with_secret("my_secret_token").await?;
///     // Manager is now ready to use with an available port
///     Ok(())
/// }
/// ```
pub async fn create_manager_with_secret(secret: &str) -> anyhow::Result<Aria2DownloadManager> {
    Aria2DownloadManager::new(
        DEFAULT_ARIA2_RPC_URL.to_string(),
        Some(secret.to_string())
    ).await
}