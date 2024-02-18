use axum::{
    debug_handler,
    extract::State,
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

use crate::{app_error::AppError, iam::upsert_iam_user, jwt, state::AppState};

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

/// Get IR as JSON
#[utoipa::path(put,
    tag="IR",
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

    let iam_client = app_state.get_iam_client();
    let codecommit_client = app_state.get_codecommit_client();

    upsert_iam_user(iam_client, &user_data.claims.sub).await?;

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
