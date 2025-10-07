use burncloud_download_types::DownloadStatus;
use crate::client::types::Aria2Status;

/// Map aria2 status string to DownloadStatus enum
pub fn map_aria2_status(aria2_status: &Aria2Status) -> DownloadStatus {
    match aria2_status.status.as_str() {
        "active" => DownloadStatus::Downloading,
        "waiting" => DownloadStatus::Waiting,
        "paused" => DownloadStatus::Paused,
        "complete" => DownloadStatus::Completed,
        "error" => {
            let error_msg = aria2_status.error_message
                .as_ref()
                .map(|s| s.clone())
                .unwrap_or_else(|| format!("Error code: {}",
                    aria2_status.error_code.as_ref().unwrap_or(&"unknown".to_string())));
            DownloadStatus::Failed(error_msg)
        }
        "removed" => DownloadStatus::Failed("Download cancelled".to_string()),
        _ => DownloadStatus::Failed(format!("Unknown status: {}", aria2_status.status)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_status(status: &str) -> Aria2Status {
        Aria2Status {
            gid: "test123".to_string(),
            status: status.to_string(),
            total_length: "1000".to_string(),
            completed_length: "500".to_string(),
            download_speed: "100".to_string(),
            upload_speed: "0".to_string(),
            files: vec![],
            error_code: None,
            error_message: None,
        }
    }

    #[test]
    fn test_map_active_status() {
        let status = create_test_status("active");
        assert_eq!(map_aria2_status(&status), DownloadStatus::Downloading);
    }

    #[test]
    fn test_map_waiting_status() {
        let status = create_test_status("waiting");
        assert_eq!(map_aria2_status(&status), DownloadStatus::Waiting);
    }

    #[test]
    fn test_map_paused_status() {
        let status = create_test_status("paused");
        assert_eq!(map_aria2_status(&status), DownloadStatus::Paused);
    }

    #[test]
    fn test_map_complete_status() {
        let status = create_test_status("complete");
        assert_eq!(map_aria2_status(&status), DownloadStatus::Completed);
    }

    #[test]
    fn test_map_error_status() {
        let mut status = create_test_status("error");
        status.error_message = Some("Connection failed".to_string());
        match map_aria2_status(&status) {
            DownloadStatus::Failed(msg) => assert_eq!(msg, "Connection failed"),
            _ => panic!("Expected Failed status"),
        }
    }
}