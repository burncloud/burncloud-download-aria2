use std::net::TcpListener;
use burncloud_download_aria2::{create_manager_with_auto_port, Aria2DownloadManager};

/// Example demonstrating automatic port conflict resolution.
/// This example shows how the library automatically finds available ports
/// when port 6800 is already in use.
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    println!("=== Port Conflict Resolution Demo ===\n");

    // Simulate port 6800 being occupied by binding to it
    println!("1. Binding to port 6800 to simulate port conflict...");
    let _port_blocker = TcpListener::bind("127.0.0.1:6800")
        .expect("Failed to bind to port 6800 for testing");
    println!("   ✓ Port 6800 is now occupied\n");

    // Create manager with automatic port conflict resolution
    println!("2. Creating Aria2DownloadManager with automatic port resolution...");
    match create_manager_with_auto_port().await {
        Ok(_manager) => {
            println!("   ✓ Manager created successfully!");
            println!("   ✓ Automatic port conflict resolution worked!");
            println!("   → The library automatically found an available port (likely 6801)");
        }
        Err(e) => {
            println!("   ✗ Failed to create manager: {}", e);
            return Err(e.into());
        }
    }

    println!("\n3. Testing with custom configuration...");

    // You can also create managers with specific URLs that will be automatically adjusted
    match Aria2DownloadManager::new(
        "http://localhost:6800/jsonrpc".to_string(),
        Some("custom_secret".to_string())
    ).await {
        Ok(_manager) => {
            println!("   ✓ Custom manager created successfully with automatic port adjustment!");
        }
        Err(e) => {
            println!("   ✗ Failed to create custom manager: {}", e);
        }
    }

    println!("\n=== Demo Complete ===");
    println!("The library now automatically handles port conflicts by:");
    println!("• Checking if the default port (6800) is available");
    println!("• If occupied, incrementing the port number until an available port is found");
    println!("• Updating both the daemon configuration and client URLs automatically");
    println!("• Providing user feedback about the port change");

    Ok(())
}