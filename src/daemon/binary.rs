use crate::error::Aria2Error;
use super::platform;
use std::path::Path;
use std::io::Cursor;

const GITHUB_DOWNLOAD_URL: &str = "https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip";
const GITEE_DOWNLOAD_URL: &str = "https://gitee.com/burncloud/aria2/raw/master/aria2-1.37.0-win-64bit-build1.zip";

/// Verify if the binary exists at the given path
pub async fn verify_binary_exists(path: &Path) -> bool {
    tokio::fs::metadata(path).await.is_ok()
}

/// Download the aria2 binary from GitHub or Gitee fallback
pub async fn download_aria2_binary(target_path: &Path) -> Result<(), Aria2Error> {
    // Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        platform::ensure_directory(parent).await?;
    }

    // Try primary source (GitHub)
    let zip_data = match download_from_url(GITHUB_DOWNLOAD_URL).await {
        Ok(data) => data,
        Err(_) => {
            // Fallback to Gitee
            download_from_url(GITEE_DOWNLOAD_URL).await
                .map_err(|e| Aria2Error::BinaryDownloadFailed(
                    format!("All sources failed. Last error: {}", e)
                ))?
        }
    };

    // Extract the binary
    extract_zip(zip_data, target_path).await?;

    // Set executable permission on Unix systems
    platform::set_executable(target_path).await?;

    Ok(())
}

/// Download binary data from a URL
async fn download_from_url(url: &str) -> Result<Vec<u8>, Aria2Error> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| Aria2Error::BinaryDownloadFailed(format!("Request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(Aria2Error::BinaryDownloadFailed(
            format!("HTTP error: {}", response.status())
        ));
    }

    let bytes = response.bytes()
        .await
        .map_err(|e| Aria2Error::BinaryDownloadFailed(format!("Failed to read response: {}", e)))?;

    Ok(bytes.to_vec())
}

/// Extract aria2c binary from zip archive
async fn extract_zip(zip_data: Vec<u8>, target_path: &Path) -> Result<(), Aria2Error> {
    let cursor = Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| Aria2Error::BinaryExtractionFailed(format!("Failed to open zip: {}", e)))?;

    // Find the aria2c binary in the archive
    let binary_name = platform::get_binary_name();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| Aria2Error::BinaryExtractionFailed(format!("Failed to read zip entry: {}", e)))?;

        // Check if this is the aria2c binary
        if let Some(name) = file.name().split('/').next_back() {
            if name == binary_name {
                // Extract to target path
                let mut out_file = std::fs::File::create(target_path)
                    .map_err(|e| Aria2Error::BinaryExtractionFailed(format!("Failed to create output file: {}", e)))?;

                std::io::copy(&mut file, &mut out_file)
                    .map_err(|e| Aria2Error::BinaryExtractionFailed(format!("Failed to extract binary: {}", e)))?;

                return Ok(());
            }
        }
    }

    Err(Aria2Error::BinaryExtractionFailed(
        format!("Binary '{}' not found in archive", binary_name)
    ))
}
