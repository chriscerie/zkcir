#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

use axum::{
    routing::{delete, get, get_service, post, put},
    Router,
};
use routes::{auth, repo};
use routes::{ir, ssh};
use routes::{profile, repos};
use state::AppState;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use utoipa::{OpenApi, ToSchema};
use utoipa_redoc::{Redoc, Servable};

mod apidoc;
mod app_error;
mod iam;
mod jwt;
mod routes;
mod state;
mod zip;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_ansi(false).init();

    let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let app_state = AppState::new(aws_config);

    let static_files =
        get_service(ServeDir::new("public").fallback(ServeFile::new("public/index.html")));

    let app = Router::new()
        .merge(Redoc::with_url("/docs", apidoc::ApiDoc::openapi()))
        .route("/auth/google", get(auth::auth_google))
        .route("/v1/profile", get(profile::get_profile))
        .route("/v1/repo", put(repo::create_repo))
        .route(
            "/v1/repo/metadata/:owner/:repo_name",
            get(repo::get_repo_metadata),
        )
        .route(
            "/v1/repo/source/:owner/:repo_name",
            get(repo::get_repo_source),
        )
        .route("/v1/repos", get(repos::list_repos))
        .route("/v1/ssh", put(ssh::create_key))
        .route("/v1/ssh", get(ssh::list_keys))
        .route("/v1/ssh/:key_id", delete(ssh::delete_key))
        .route("/v1/ir", post(ir::compile_to_ir))
        .route("/v1/ir/:repo_name/:circuit_version", get(ir::get_ir))
        .route("/v1/ir/metadata/list", get(ir::list_irs_metadata))
        .route("/v1/ir/versions/:repo_name", get(ir::list_ir_versions))
        .route(
            "/v1/ir/source/:owner/:repo_name/:circuit_version",
            get(ir::get_ir_source),
        )
        .with_state(app_state)
        .fallback(static_files)
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_origin(Any),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(ToSchema)]
#[allow(unused)]
pub struct UnauthorizedResponse {
    message: String,
}
