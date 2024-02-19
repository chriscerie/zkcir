use axum::http::StatusCode;
use tempfile::TempDir;

use crate::app_error::AppError;

pub fn clone_repo(clone_url: &str) -> Result<TempDir, AppError> {
    let dir = tempfile::tempdir().map_err(|e| {
        tracing::error!("Failed to create temporary directory: {:#?}", e);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create temporary directory".to_string(),
        )
    })?;

    let output = std::process::Command::new("git")
        .arg("clone")
        .arg(clone_url)
        .arg(dir.path())
        .output()
        .map_err(|e| {
            tracing::error!("Failed to execute `git clone`: {:#?}", e);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to clone repository".to_string(),
            )
        })?;

    if !output.status.success() {
        tracing::error!("`git clone` failed: {:#?}", output);
        return Err(AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to clone repository".to_string(),
        ));
    }

    std::fs::remove_dir_all(dir.path().join(".git")).map_err(|e| {
        tracing::error!("Failed to remove .git directory: {:#?}", e);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to remove .git directory".to_string(),
        )
    })?;

    Ok(dir)
}
