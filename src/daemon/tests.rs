/// Unit tests for aria2 daemon components
use super::*;
use std::path::PathBuf;

#[cfg(test)]
mod platform_tests {
    use super::*;

    #[test]
    fn test_get_binary_dir() {
        let dir = platform::get_binary_dir();
        assert!(!dir.to_string_lossy().is_empty());

        #[cfg(target_os = "windows")]
        assert!(dir.to_string_lossy().contains("BurnCloud"));

        #[cfg(target_os = "linux")]
        assert!(dir.to_string_lossy().contains(".burncloud"));
    }

    #[test]
    fn test_get_binary_name() {
        let name = platform::get_binary_name();
        assert!(!name.is_empty());

        #[cfg(target_os = "windows")]
        assert_eq!(name, "aria2c.exe");

        #[cfg(not(target_os = "windows"))]
        assert_eq!(name, "aria2c");
    }

    #[test]
    fn test_get_binary_path() {
        let path = platform::get_binary_path();
        assert!(!path.to_string_lossy().is_empty());
        assert!(path.to_string_lossy().ends_with(platform::get_binary_name()));
    }

    #[tokio::test]
    async fn test_ensure_directory_creates_new_dir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("test_subdir");

        assert!(!new_dir.exists());
        platform::ensure_directory(&new_dir).await.unwrap();
        assert!(new_dir.exists());
    }

    #[tokio::test]
    async fn test_ensure_directory_existing_dir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Should not error on existing directory
        platform::ensure_directory(path).await.unwrap();
        assert!(path.exists());
    }
}

#[cfg(test)]
mod binary_tests {
    use super::*;

    #[tokio::test]
    async fn test_verify_binary_exists_missing() {
        let path = PathBuf::from("nonexistent_binary_path_12345.exe");
        assert!(!binary::verify_binary_exists(&path).await);
    }

    #[tokio::test]
    async fn test_verify_binary_exists_present() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        assert!(binary::verify_binary_exists(path).await);
    }
}

#[cfg(test)]
mod process_tests {
    use super::*;

    #[test]
    fn test_process_config_creation() {
        let config = process::ProcessConfig {
            rpc_port: 6800,
            rpc_secret: "test_secret".to_string(),
            download_dir: PathBuf::from("/tmp"),
            session_file: PathBuf::from("/tmp/aria2.session"),
            max_restart_attempts: 5,
        };

        assert_eq!(config.rpc_port, 6800);
        assert_eq!(config.rpc_secret, "test_secret");
        assert_eq!(config.max_restart_attempts, 5);
    }

    #[tokio::test]
    async fn test_restart_counter_increment() {
        let config = process::ProcessConfig {
            rpc_port: 6800,
            rpc_secret: "test".to_string(),
            download_dir: PathBuf::from("/tmp"),
            session_file: PathBuf::from("/tmp/aria2.session"),
            max_restart_attempts: 10,
        };

        let handle = process::ProcessHandle::new(
            PathBuf::from("test_binary"),
            config,
        );

        // Test increment
        assert_eq!(handle.increment_restart_count().await, 1);
        assert_eq!(handle.increment_restart_count().await, 2);
        assert_eq!(handle.increment_restart_count().await, 3);

        // Test reset
        handle.reset_restart_count().await;
        assert_eq!(handle.increment_restart_count().await, 1);
    }

    #[tokio::test]
    async fn test_max_restart_attempts() {
        let config = process::ProcessConfig {
            rpc_port: 6800,
            rpc_secret: "test".to_string(),
            download_dir: PathBuf::from("/tmp"),
            session_file: PathBuf::from("/tmp/aria2.session"),
            max_restart_attempts: 5,
        };

        let handle = process::ProcessHandle::new(
            PathBuf::from("test_binary"),
            config,
        );

        assert_eq!(handle.max_restart_attempts(), 5);
    }

    #[tokio::test]
    async fn test_process_not_running_initially() {
        let config = process::ProcessConfig {
            rpc_port: 6800,
            rpc_secret: "test".to_string(),
            download_dir: PathBuf::from("/tmp"),
            session_file: PathBuf::from("/tmp/aria2.session"),
            max_restart_attempts: 10,
        };

        let handle = process::ProcessHandle::new(
            PathBuf::from("nonexistent_binary"),
            config,
        );

        assert!(!handle.is_running().await);
    }
}

#[cfg(test)]
mod daemon_config_tests {
    use super::*;

    #[test]
    fn test_daemon_config_default() {
        let config = orchestrator::DaemonConfig::default();

        assert_eq!(config.rpc_port, 6800);
        assert_eq!(config.rpc_secret, "burncloud");
        assert_eq!(config.max_restart_attempts, 10);
        assert_eq!(config.health_check_interval, std::time::Duration::from_secs(10));
    }

    #[test]
    fn test_daemon_config_custom() {
        let config = orchestrator::DaemonConfig {
            rpc_port: 7800,
            rpc_secret: "custom_secret".to_string(),
            download_dir: PathBuf::from("/custom/path"),
            session_file: PathBuf::from("/custom/path/aria2.session"),
            max_restart_attempts: 3,
            health_check_interval: std::time::Duration::from_secs(5),
        };

        assert_eq!(config.rpc_port, 7800);
        assert_eq!(config.rpc_secret, "custom_secret");
        assert_eq!(config.max_restart_attempts, 3);
        assert_eq!(config.health_check_interval, std::time::Duration::from_secs(5));
    }

    #[test]
    fn test_daemon_config_clone() {
        let config1 = orchestrator::DaemonConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1.rpc_port, config2.rpc_port);
        assert_eq!(config1.rpc_secret, config2.rpc_secret);
        assert_eq!(config1.max_restart_attempts, config2.max_restart_attempts);
    }
}
