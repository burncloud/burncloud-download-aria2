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