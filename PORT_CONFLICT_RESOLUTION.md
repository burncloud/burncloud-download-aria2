# Port Conflict Resolution

As of this update, the BurnCloud Aria2 Download Manager now includes automatic port conflict resolution.

## Problem
Previously, if port 6800 was already in use (by another aria2 instance, web server, or any other application), the library would fail to start with a port binding error.

## Solution
The library now automatically detects port conflicts and finds the next available port:

1. **Automatic Detection**: When creating a manager, the library checks if the configured port is available
2. **Incremental Search**: If the port is occupied, it searches for the next available port starting from the current port + 1
3. **URL Updates**: Both the daemon configuration and client RPC URLs are automatically updated to use the new port
4. **User Feedback**: The library prints a message informing the user about the port change

## Usage

### Simple Usage (Recommended)
```rust
use burncloud_download_aria2::create_manager_with_auto_port;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This will automatically find an available port starting from 6800
    let manager = create_manager_with_auto_port().await?;
    Ok(())
}
```

### With Custom Secret
```rust
use burncloud_download_aria2::create_manager_with_secret;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_manager_with_secret("my_secret").await?;
    Ok(())
}
```

### Manual Configuration (Still Supported)
```rust
use burncloud_download_aria2::Aria2DownloadManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Even with manual configuration, port conflict resolution is automatic
    let manager = Aria2DownloadManager::new(
        "http://localhost:6800/jsonrpc".to_string(),
        Some("secret".to_string())
    ).await?;
    Ok(())
}
```

## Example Output
When port 6800 is occupied, you'll see output like:
```
Port 6800 is occupied. Using port 6801 instead.
Updated RPC URL to: http://localhost:6801/jsonrpc
```

## Implementation Details

### Port Checking
- Uses `TcpListener::bind()` to check if a port is available for binding
- Uses async TCP connection attempts to verify ports are not actively in use
- Searches through ports sequentially from the starting port to 65535

### Error Handling
- If no available ports are found (extremely unlikely), returns a clear error message
- Maintains all existing error handling for other failure scenarios

### Backwards Compatibility
- All existing APIs continue to work unchanged
- No breaking changes to existing code
- The feature is enabled by default and requires no configuration

## Testing

The implementation includes comprehensive tests:
- Port availability checking
- Port conflict simulation
- URL parsing and updating
- Integration with the full manager lifecycle

To run the demo:
```bash
cargo run --example port_conflict_demo
```