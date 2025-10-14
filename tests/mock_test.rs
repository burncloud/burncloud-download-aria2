use burncloud_download_aria2::poller::aggregator::ProgressAggregator;
use serde_json::json;

// Removed all status mapping tests since mapper module was deleted
// Only keeping progress aggregator tests as they work with raw JSON

#[test]
fn test_progress_aggregator_single_file() {
    let status = json!({
        "gid": "test",
        "status": "active",
        "totalLength": "1000",
        "completedLength": "500",
        "downloadSpeed": "100",
        "uploadSpeed": "0",
        "files": [
            {
                "index": "1",
                "path": "/file1",
                "length": "1000",
                "completedLength": "500",
                "selected": "true"
            }
        ]
    });

    let progress = ProgressAggregator::aggregate(&status);
    assert_eq!(progress.total_bytes, 1000);
    assert_eq!(progress.downloaded_bytes, 500);
}

#[test]
fn test_progress_aggregator_multiple_files() {
    let status = json!({
        "gid": "test",
        "status": "active",
        "totalLength": "5000",
        "completedLength": "2500",
        "downloadSpeed": "100",
        "uploadSpeed": "0",
        "files": [
            {
                "index": "1",
                "path": "/file1",
                "length": "1000",
                "completedLength": "500",
                "selected": "true"
            },
            {
                "index": "2",
                "path": "/file2",
                "length": "2000",
                "completedLength": "1000",
                "selected": "true"
            },
            {
                "index": "3",
                "path": "/file3",
                "length": "2000",
                "completedLength": "1000",
                "selected": "true"
            }
        ]
    });

    let progress = ProgressAggregator::aggregate(&status);
    assert_eq!(progress.total_bytes, 5000);
    assert_eq!(progress.downloaded_bytes, 2500);
}

#[test]
fn test_progress_aggregator_empty_files() {
    let status = json!({
        "gid": "test",
        "status": "active",
        "totalLength": "0",
        "completedLength": "0",
        "downloadSpeed": "0",
        "uploadSpeed": "0",
        "files": []
    });

    let progress = ProgressAggregator::aggregate(&status);
    assert_eq!(progress.total_bytes, 0);
    assert_eq!(progress.downloaded_bytes, 0);
}