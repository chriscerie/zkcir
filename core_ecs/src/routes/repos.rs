use axum::{debug_handler, extract::State, response::IntoResponse, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use common::targets::TargetFramework;
use utoipa::ToSchema;

use crate::{app_error::AppError, jwt, state::AppState};

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct Repository {
    /// Name without the user's namespace
    name: String,

    /// <sub>.<name>
    full_name: String,

    /// Last modified date in seconds since Unix epoch
    last_modified_date: Option<f64>,

    framework: Option<TargetFramework>,
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

    let codecommit_client = app_state.codecommit_client;

    let repos = codecommit_client.list_repositories().send().await?;

    let mut owned_repos = Vec::new();

    for repo in repos.repositories.unwrap_or_default() {
        let repo_name = repo.repository_name.unwrap_or_default();

        if user_data.claims.sub == repo_name.split('.').next().unwrap_or_default() {
            let metadata = codecommit_client
                .get_repository()
                .repository_name(&repo_name)
                .send()
                .await?;

            let last_modified_date = metadata
                .repository_metadata()
                .and_then(|metadata| metadata.last_modified_date);

            owned_repos.push(Repository {
                name: repo_name.split('.').skip(1).collect(),
                full_name: repo_name,
                last_modified_date: last_modified_date.map(|date| date.as_secs_f64()),

                // TODO: Get actual framework
                framework: Some(TargetFramework::Plonky2),
            });
        }
    }

    Ok(Json(ListReposResponse { repos: owned_repos }))
}
