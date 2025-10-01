use crate::error::Aria2Error;
use std::path::{Path, PathBuf};

/// Get the platform-specific directory for storing aria2 binary
#[cfg(target_os = "windows")]
pub fn get_binary_dir() -> PathBuf {
    let localappdata = std::env::var("LOCALAPPDATA")
        .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".to_string());
    PathBuf::from(localappdata).join("BurnCloud")
}

#[cfg(target_os = "linux")]
pub fn get_binary_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".burncloud")
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn get_binary_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".burncloud")
}

/// Get the platform-specific binary name
#[cfg(target_os = "windows")]
pub fn get_binary_name() -> &'static str {
    "aria2c.exe"
}

#[cfg(not(target_os = "windows"))]
pub fn get_binary_name() -> &'static str {
    "aria2c"
}

/// Get the full path to the aria2 binary
pub fn get_binary_path() -> PathBuf {
    get_binary_dir().join(get_binary_name())
}

/// Ensure the directory exists, creating it if necessary
pub async fn ensure_directory(path: &Path) -> Result<(), Aria2Error> {
    if !path.exists() {
        tokio::fs::create_dir_all(path).await?;
    }
    Ok(())
}

/// Set executable permissions on Unix systems
#[cfg(unix)]
pub async fn set_executable(path: &Path) -> Result<(), Aria2Error> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = tokio::fs::metadata(path).await?.permissions();
    perms.set_mode(0o755);
    tokio::fs::set_permissions(path, perms).await?;
    Ok(())
}

#[cfg(not(unix))]
pub async fn set_executable(_path: &Path) -> Result<(), Aria2Error> {
    // No-op on Windows
    Ok(())
}
