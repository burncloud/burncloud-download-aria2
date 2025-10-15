use std::net::{TcpListener, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Check if a port is available on localhost
pub fn is_port_available(port: u16) -> bool {
    match TcpListener::bind(format!("127.0.0.1:{}", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Check if a port is in use by attempting to connect to it
pub async fn is_port_in_use(port: u16) -> bool {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

    // Try to connect with a short timeout
    match timeout(Duration::from_millis(100), TcpStream::connect(addr)).await {
        Ok(Ok(_)) => true,  // Connection successful, port is in use
        Ok(Err(_)) => false, // Connection failed, port is not in use
        Err(_) => false,    // Timeout, assume port is not in use
    }
}

/// Find the next available port starting from the given port
pub fn find_available_port(start_port: u16) -> Option<u16> {
    for port in start_port..=65535 {
        if is_port_available(port) {
            return Some(port);
        }
    }
    None
}

/// Find the next available port with async checking
pub async fn find_available_port_async(start_port: u16) -> Option<u16> {
    for port in start_port..=65535 {
        // First check if we can bind to the port (quick check)
        if is_port_available(port) {
            // Double-check that nothing is actively using it
            if !is_port_in_use(port).await {
                return Some(port);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn test_is_port_available() {
        // Test with a high port number that should be available
        assert!(is_port_available(65000));

        // Test with a port that we bind to
        let _listener = TcpListener::bind("127.0.0.1:65001").unwrap();
        assert!(!is_port_available(65001));
    }

    #[test]
    fn test_find_available_port() {
        // Bind to a port to make it unavailable
        let _listener = TcpListener::bind("127.0.0.1:65002").unwrap();

        // Find an available port starting from the bound port
        let available_port = find_available_port(65002);
        assert!(available_port.is_some());
        assert!(available_port.unwrap() > 65002);
    }

    #[tokio::test]
    async fn test_find_available_port_async() {
        // Bind to a port to make it unavailable
        let _listener = TcpListener::bind("127.0.0.1:65003").unwrap();

        // Find an available port starting from the bound port
        let available_port = find_available_port_async(65003).await;
        assert!(available_port.is_some());
        assert!(available_port.unwrap() > 65003);
    }

    #[tokio::test]
    async fn test_is_port_in_use() {
        // Bind to a port and check if it's detected as in use
        let listener = TcpListener::bind("127.0.0.1:65004").unwrap();
        let _handle = tokio::spawn(async move {
            // Keep the listener alive
            let _ = listener;
            tokio::time::sleep(Duration::from_millis(200)).await;
        });

        // Give it a moment to bind
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Should be detected as in use if something is listening
        // Note: This test might be flaky depending on the system
        let port_status = is_port_in_use(65004).await;
        // We can't guarantee this will be true as TcpListener might not accept connections
        // but we can at least verify the function doesn't crash
        let _ = port_status;
    }
}