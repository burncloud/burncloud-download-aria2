use serde_json::Value;

/// Aggregate progress for multi-file downloads
/// Works directly with JSON values for real-time data access
pub struct ProgressAggregator;

impl ProgressAggregator {
    pub fn aggregate(status: &Value) -> AggregatedProgress {
        // Extract files array from JSON
        let empty_files = Vec::new();
        let files = status
            .get("files")
            .and_then(|f| f.as_array())
            .unwrap_or(&empty_files);

        let total_bytes: u64 = files.iter()
            .filter_map(|f| {
                f.get("length")
                    .and_then(|l| l.as_str())
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .sum();

        let downloaded_bytes: u64 = files.iter()
            .filter_map(|f| {
                f.get("completedLength")
                    .and_then(|l| l.as_str())
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .sum();

        AggregatedProgress {
            total_bytes,
            downloaded_bytes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AggregatedProgress {
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_aggregate_multi_file() {
        let status = json!({
            "gid": "test",
            "status": "active",
            "totalLength": "3000",
            "completedLength": "1500",
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
                }
            ]
        });

        let progress = ProgressAggregator::aggregate(&status);
        assert_eq!(progress.total_bytes, 3000);
        assert_eq!(progress.downloaded_bytes, 1500);
    }

    #[test]
    fn test_aggregate_empty_files() {
        let status = json!({
            "gid": "test",
            "status": "waiting",
            "files": []
        });

        let progress = ProgressAggregator::aggregate(&status);
        assert_eq!(progress.total_bytes, 0);
        assert_eq!(progress.downloaded_bytes, 0);
    }

    #[test]
    fn test_aggregate_no_files_field() {
        let status = json!({
            "gid": "test",
            "status": "waiting"
        });

        let progress = ProgressAggregator::aggregate(&status);
        assert_eq!(progress.total_bytes, 0);
        assert_eq!(progress.downloaded_bytes, 0);
    }
}