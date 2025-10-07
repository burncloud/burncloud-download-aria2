use burncloud_download_aria2::client::types::{Aria2Status, Aria2File};
use burncloud_download_aria2::manager::mapper;
use burncloud_download_aria2::poller::aggregator::ProgressAggregator;
use burncloud_download_types::DownloadStatus;

#[test]
fn test_state_mapping_all_statuses() {
    let test_cases = vec![
        ("active", DownloadStatus::Downloading),
        ("waiting", DownloadStatus::Waiting),
        ("paused", DownloadStatus::Paused),
        ("complete", DownloadStatus::Completed),
    ];

    for (aria2_status_str, expected_status) in test_cases {
        let status = create_test_status(aria2_status_str);
        let mapped = mapper::map_aria2_status(&status);
        assert_eq!(mapped, expected_status, "Failed for status: {}", aria2_status_str);
    }
}

#[test]
fn test_error_status_with_message() {
    let mut status = create_test_status("error");
    status.error_message = Some("Network error".to_string());
    status.error_code = Some("1".to_string());

    match mapper::map_aria2_status(&status) {
        DownloadStatus::Failed(msg) => {
            assert_eq!(msg, "Network error");
        }
        _ => panic!("Expected Failed status"),
    }
}

#[test]
fn test_error_status_without_message() {
    let mut status = create_test_status("error");
    status.error_code = Some("5".to_string());

    match mapper::map_aria2_status(&status) {
        DownloadStatus::Failed(msg) => {
            assert!(msg.contains("Error code: 5"));
        }
        _ => panic!("Expected Failed status"),
    }
}

#[test]
fn test_removed_status() {
    let status = create_test_status("removed");
    match mapper::map_aria2_status(&status) {
        DownloadStatus::Failed(msg) => {
            assert_eq!(msg, "Download cancelled");
        }
        _ => panic!("Expected Failed status"),
    }
}

#[test]
fn test_unknown_status() {
    let status = create_test_status("unknown_status");
    match mapper::map_aria2_status(&status) {
        DownloadStatus::Failed(msg) => {
            assert!(msg.contains("Unknown status"));
        }
        _ => panic!("Expected Failed status"),
    }
}

#[test]
fn test_progress_aggregator_single_file() {
    let status = Aria2Status {
        gid: "test".to_string(),
        status: "active".to_string(),
        total_length: "1000".to_string(),
        completed_length: "500".to_string(),
        download_speed: "100".to_string(),
        upload_speed: "0".to_string(),
        files: vec![
            Aria2File {
                index: "1".to_string(),
                path: "/file1".to_string(),
                length: "1000".to_string(),
                completed_length: "500".to_string(),
                selected: "true".to_string(),
            },
        ],
        error_code: None,
        error_message: None,
    };

    let progress = ProgressAggregator::aggregate(&status);
    assert_eq!(progress.total_bytes, 1000);
    assert_eq!(progress.downloaded_bytes, 500);
}

#[test]
fn test_progress_aggregator_multiple_files() {
    let status = Aria2Status {
        gid: "test".to_string(),
        status: "active".to_string(),
        total_length: "5000".to_string(),
        completed_length: "2500".to_string(),
        download_speed: "100".to_string(),
        upload_speed: "0".to_string(),
        files: vec![
            Aria2File {
                index: "1".to_string(),
                path: "/file1".to_string(),
                length: "1000".to_string(),
                completed_length: "500".to_string(),
                selected: "true".to_string(),
            },
            Aria2File {
                index: "2".to_string(),
                path: "/file2".to_string(),
                length: "2000".to_string(),
                completed_length: "1000".to_string(),
                selected: "true".to_string(),
            },
            Aria2File {
                index: "3".to_string(),
                path: "/file3".to_string(),
                length: "2000".to_string(),
                completed_length: "1000".to_string(),
                selected: "true".to_string(),
            },
        ],
        error_code: None,
        error_message: None,
    };

    let progress = ProgressAggregator::aggregate(&status);
    assert_eq!(progress.total_bytes, 5000);
    assert_eq!(progress.downloaded_bytes, 2500);
}

#[test]
fn test_progress_aggregator_empty_files() {
    let status = Aria2Status {
        gid: "test".to_string(),
        status: "active".to_string(),
        total_length: "0".to_string(),
        completed_length: "0".to_string(),
        download_speed: "0".to_string(),
        upload_speed: "0".to_string(),
        files: vec![],
        error_code: None,
        error_message: None,
    };

    let progress = ProgressAggregator::aggregate(&status);
    assert_eq!(progress.total_bytes, 0);
    assert_eq!(progress.downloaded_bytes, 0);
}

// Helper function
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