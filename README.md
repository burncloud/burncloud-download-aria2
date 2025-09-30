# burncloud-download-aria2

Aria2 backend implementation for BurnCloud download manager.

## Features

- **Custom JSON-RPC client** for aria2 communication (no external JSON-RPC dependencies)
- **Multi-protocol support**: HTTP/HTTPS, BitTorrent, Metalink, and Magnet downloads
- **Real-time progress tracking** with 1-second polling intervals
- **Multi-file download aggregation** for torrents and metalinks
- **Full DownloadManager trait implementation** for seamless integration

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
burncloud-download-aria2 = { path = "../burncloud-download-aria2" }
```

## Prerequisites

This crate requires a running aria2 daemon with RPC interface enabled.

### Starting aria2 daemon

```bash
# Basic setup (no authentication)
aria2c --enable-rpc --rpc-listen-port=6800

# With authentication
aria2c --enable-rpc --rpc-secret=mysecret --rpc-listen-port=6800
```

### Installing aria2

- **Ubuntu/Debian**: `sudo apt-get install aria2`
- **macOS**: `brew install aria2`
- **Windows**: Download from [aria2 releases](https://github.com/aria2/aria2/releases)

## Usage

```rust
use burncloud_download_aria2::Aria2DownloadManager;
use burncloud_download::DownloadManager;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create manager (connects to local aria2 daemon)
    let manager = Aria2DownloadManager::new(
        "http://localhost:6800/jsonrpc".to_string(),
        None // or Some("your-secret".to_string())
    );

    // Add a download
    let task_id = manager.add_download(
        "https://example.com/file.zip".to_string(),
        PathBuf::from("/downloads/file.zip")
    ).await?;

    // Check progress
    let progress = manager.get_progress(task_id).await?;
    println!("Downloaded: {} / {} bytes",
        progress.downloaded_bytes,
        progress.total_bytes.unwrap_or(0)
    );

    // Pause download
    manager.pause_download(task_id).await?;

    // Resume download
    manager.resume_download(task_id).await?;

    // Cancel download
    manager.cancel_download(task_id).await?;

    Ok(())
}
```

## Supported Download Types

### HTTP/HTTPS Downloads
```rust
let task_id = manager.add_download(
    "https://example.com/file.zip".to_string(),
    PathBuf::from("/downloads/file.zip")
).await?;
```

### Magnet Links
```rust
let task_id = manager.add_download(
    "magnet:?xt=urn:btih:...".to_string(),
    PathBuf::from("/downloads/torrent")
).await?;
```

### Torrent Files
```rust
let task_id = manager.add_download(
    "https://example.com/file.torrent".to_string(),
    PathBuf::from("/downloads/torrent")
).await?;
```

### Metalink Files
```rust
let task_id = manager.add_download(
    "https://example.com/file.metalink".to_string(),
    PathBuf::from("/downloads/files")
).await?;
```

## Architecture

### Component Structure

```
burncloud-download-aria2/
├── src/
│   ├── client/           # JSON-RPC client
│   │   ├── mod.rs        # Aria2Client implementation
│   │   └── types.rs      # JSON-RPC request/response types
│   ├── manager/          # DownloadManager implementation
│   │   ├── mod.rs        # Aria2DownloadManager
│   │   ├── state.rs      # Internal state management
│   │   └── mapper.rs     # State mapping (aria2 → DownloadStatus)
│   ├── poller/           # Progress tracking
│   │   ├── mod.rs        # ProgressPoller (1-second interval)
│   │   └── aggregator.rs # Multi-file progress aggregation
│   ├── error.rs          # Error types
│   └── lib.rs            # Public API
└── tests/
    ├── integration_test.rs # Integration tests (requires aria2 daemon)
    └── mock_test.rs        # Unit tests
```

### State Mapping

| aria2 Status | DownloadStatus |
|-------------|----------------|
| `active` | `Downloading` |
| `waiting` | `Waiting` |
| `paused` | `Paused` |
| `complete` | `Completed` |
| `error` | `Failed(error_message)` |
| `removed` | `Failed("Download cancelled")` |

### Progress Polling

The `ProgressPoller` runs in the background and updates task status every 1 second:
- Polls all active tasks from aria2
- Updates internal state cache
- Maps aria2 status to `DownloadStatus`
- Aggregates progress for multi-file downloads

## Testing

### Unit Tests
```bash
# Run all unit tests
cargo test --lib

# Run specific test module
cargo test --lib mapper::tests
```

### Integration Tests
Integration tests require a running aria2 daemon:

```bash
# Start aria2
aria2c --enable-rpc --rpc-listen-port=6800

# Run integration tests (ignored by default)
cargo test -- --ignored
```

### Mock Tests
```bash
cargo test --test mock_test
```

## Error Handling

The crate defines `Aria2Error` for all error conditions:

- `RpcError(code, message)` - JSON-RPC errors from aria2
- `TransportError` - Network/HTTP errors
- `SerializationError` - JSON parsing errors
- `DaemonUnavailable` - Cannot connect to aria2
- `TaskNotFound` - Task doesn't exist in aria2
- `InvalidUrl` - Unsupported URL scheme
- `InvalidPath` - Invalid file path

All errors automatically convert to `anyhow::Error` for easy propagation.

## Performance Considerations

- **Progress polling**: 1 RPC call per second per active task
- **State caching**: Uses `RwLock` for concurrent read access
- **Memory overhead**: In-memory task state mapping only
- **No persistence**: State lost on manager drop

## Limitations

- No automatic recovery from aria2 daemon restarts
- No persistent state across application restarts
- Individual file progress not exposed (aggregate only for multi-file downloads)
- No aria2 configuration options exposed (uses defaults)

## Future Enhancements

- Event-based notifications instead of polling
- Persistent state storage
- Bulk status queries optimization
- Per-download aria2 options configuration
- Connection pooling for high-frequency RPC calls

## License

MIT

## References

- [aria2 JSON-RPC Documentation](https://aria2.github.io/manual/en/html/aria2c.html#rpc-interface)
- [BurnCloud Download Manager Trait](../burncloud-download/src/traits/manager.rs)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)