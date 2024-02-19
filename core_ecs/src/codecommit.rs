use axum::http::StatusCode;

use crate::app_error::AppError;

pub async fn get_http_clone_url(
    codecommit_client: &aws_sdk_codecommit::Client,
    repo_full_name: &str,
) -> Result<String, AppError> {
    let Some(metadata) = codecommit_client
        .get_repository()
        .repository_name(repo_full_name)
        .send()
        .await?
        .repository_metadata
    else {
        return Err(AppError::new(
            StatusCode::NOT_FOUND,
            "Repository not found".to_string(),
        ));
    };

    metadata.clone_url_http.ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Repository does not have a clone url".to_string(),
        )
    })
}
