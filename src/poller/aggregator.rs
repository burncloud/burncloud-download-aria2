use crate::client::types::Aria2Status;

/// Aggregate progress for multi-file downloads
pub struct ProgressAggregator;

impl ProgressAggregator {
    pub fn aggregate(status: &Aria2Status) -> AggregatedProgress {
        let total_bytes: u64 = status.files.iter()
            .filter_map(|f| f.length.parse::<u64>().ok())
            .sum();

        let downloaded_bytes: u64 = status.files.iter()
            .filter_map(|f| f.completed_length.parse::<u64>().ok())
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
    use crate::client::types::{Aria2Status, Aria2File};

    #[test]
    fn test_aggregate_multi_file() {
        let status = Aria2Status {
            gid: "test".to_string(),
            status: "active".to_string(),
            total_length: "3000".to_string(),
            completed_length: "1500".to_string(),
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
            ],
            error_code: None,
            error_message: None,
        };

        let progress = ProgressAggregator::aggregate(&status);
        assert_eq!(progress.total_bytes, 3000);
        assert_eq!(progress.downloaded_bytes, 1500);
    }
}