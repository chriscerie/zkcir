use axum::{debug_handler, extract::State, response::IntoResponse, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use utoipa::ToSchema;

use crate::{app_error::AppError, jwt, state::AppState};

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct Repository {
    /// Name without the user's namespace
    name: String,

    /// <sub>.<name>
    full_name: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ListReposResponse {
    repos: Vec<Repository>,
}

/// List repositories
#[utoipa::path(get,
    tag="Repos",
    path="/v1/repos",
    responses(
        (status = 200, description = "Got IRs", body = ListIrsMetadataResponse),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn list_repos(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let token = bearer
        .token()
        .to_string()
        .trim_start_matches("Bearer ")
        .to_string();
    let user_data = jwt::get_user_claims(&token)?;

    let codecommit_client = app_state.get_codecommit_client();

    let repos = codecommit_client.list_repositories().send().await?;

    let owned_repos = repos
        .repositories
        .unwrap_or_default()
        .into_iter()
        .filter_map(|repo| {
            let repo_name = repo.repository_name.unwrap_or_default();

            if user_data.claims.sub == repo_name.split('.').next().unwrap_or_default() {
                return Some(Repository {
                    name: repo_name.split('.').skip(1).collect(),
                    full_name: repo_name,
                });
            }
            None
        })
        .collect::<Vec<_>>();

    Ok(Json(ListReposResponse { repos: owned_repos }))
}
