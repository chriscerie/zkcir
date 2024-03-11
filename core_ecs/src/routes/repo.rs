use std::{fs::File, io::Read};

use aws_sdk_codecommit::operation::get_branch::GetBranchError;
use axum::{
    body::Body,
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    app_error::AppError, codecommit::get_http_clone_url, git::clone_repo, iam::upsert_iam_user,
    jwt, state::AppState, zip::zip_path,
};

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreateRepoResponse {
    repo_name: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct CreateRepoPayload {
    /// Repository name. Cannot collide with existing repositories in the same user's namespace
    repo_name: String,

    /// Repository description
    description: Option<String>,

    /// Repository visibility
    visibility: Option<String>,
}

/// Initiate compilation to IR
#[utoipa::path(put,
    tag="Repo",
    path="/v1/repo",
    responses(
        (status = 201, description = "Create repository", body = CreateRepoResponse),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn create_repo(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Json(payload): Json<CreateRepoPayload>,
) -> Result<impl IntoResponse, AppError> {
    let token = bearer
        .token()
        .to_string()
        .trim_start_matches("Bearer ")
        .to_string();
    let user_data = jwt::get_user_claims(&token)?;

    let codecommit_client = app_state.codecommit_client;

    upsert_iam_user(&app_state.iam_client, &user_data.claims.sub).await?;

    let repo_name = format!("{}.{}", &user_data.claims.sub, &payload.repo_name);

    codecommit_client
        .create_repository()
        .repository_name(&repo_name)
        .set_repository_description(payload.description)
        .send()
        .await?;

    Response::builder()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&CreateRepoResponse { repo_name }).map_err(AppError::from)?)
        .map_err(AppError::from)
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetRepoMetadataResponse {
    /// Description of repository
    description: String,

    /// Url used to clone repo over SSH
    clone_url_ssh: String,

    /// Latest commit of the main branch
    // For simplicity we only support the main branch
    latest_commit_id: Option<String>,
}

/// Get repository metadata
#[utoipa::path(get,
    tag="Repo",
    path="/v1/repo/metadata/{owner}/{repo_name}",
    responses(
        (status = 200, description = "Got repository", body = GetRepoMetadataResponse),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn get_repo_metadata(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let token = bearer
        .token()
        .to_string()
        .trim_start_matches("Bearer ")
        .to_string();
    let user_data = jwt::get_user_claims(&token)?;

    if user_data.claims.sub != owner {
        return Err(AppError::new(
            StatusCode::FORBIDDEN,
            "You do not have access to this repository".to_string(),
        ));
    }

    let repo_full_name = format!("{owner}.{repo_name}");

    let codecommit_client = app_state.codecommit_client;

    let Some(metadata) = codecommit_client
        .get_repository()
        .repository_name(&repo_full_name)
        .send()
        .await?
        .repository_metadata
    else {
        return Err(AppError::new(
            StatusCode::NOT_FOUND,
            "Repository not found".to_string(),
        ));
    };

    let branch = match codecommit_client
        .get_branch()
        .repository_name(&repo_full_name)
        .branch_name("main")
        .send()
        .await
    {
        Ok(response) => Some(response.branch.unwrap()),
        Err(aws_sdk_codecommit::error::SdkError::ServiceError(err)) => {
            if let GetBranchError::BranchDoesNotExistException(_) = err.err() {
                None
            } else {
                tracing::error!("Failed to get repository's branch: {:#?}", err.err());
                return Err(AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to get repository's branch".to_string(),
                ));
            }
        }
        Err(err) => return Err(AppError::from(err)),
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_string(&GetRepoMetadataResponse {
                description: metadata.repository_description.unwrap_or_default(),
                clone_url_ssh: metadata.clone_url_ssh.unwrap_or_default(),
                latest_commit_id: branch.and_then(|b| b.commit_id),
            })
            .map_err(AppError::from)?,
        )
        .map_err(AppError::from)
}

/// Get repo source as zip
#[utoipa::path(get,
    tag="Repo",
    path="/v1/repo/source/{owner}/{repo_name}",
    responses(
        (status = 200, description = "Got source", body = Vec<u8>, content_type = "application/zip"),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Invalid circuit id", body = String),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn get_repo_source(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let token = bearer
        .token()
        .to_string()
        .trim_start_matches("Bearer ")
        .to_string();
    let user_data = jwt::get_user_claims(&token)?;

    if user_data.claims.sub != owner {
        return Err(AppError::new(
            StatusCode::FORBIDDEN,
            "You do not have access to this repository".to_string(),
        ));
    }

    let repo_full_name = format!("{owner}.{repo_name}");

    let clone_url = get_http_clone_url(&app_state.codecommit_client, &repo_full_name).await?;
    let repo_dir = clone_repo(&clone_url)?;

    let zipped_source = zip_path(repo_dir.path())?;

    let mut zipped_source_file = File::open(zipped_source.1).map_err(|e| {
        tracing::error!("Failed to open zip file: {e}");
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal error".to_string(),
        )
    })?;

    let mut source_buffer = Vec::new();

    zipped_source_file
        .read_to_end(&mut source_buffer)
        .map_err(|e| {
            tracing::error!("Failed to read zip file: {e}");
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?;

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/zip")
        .body(Body::from(source_buffer))
        .map_err(AppError::from)
}
