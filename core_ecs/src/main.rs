#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

use axum::{
    routing::{get, get_service, post},
    Router,
};
use routes::auth;
use routes::ir;
use routes::profile;
use state::AppState;
use tower_http::services::{ServeDir, ServeFile};
use utoipa::{OpenApi, ToSchema};
use utoipa_redoc::{Redoc, Servable};

mod apidoc;
mod app_error;
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
        .route("/v1/ir", post(ir::compile_to_ir))
        .route("/v1/ir/:repo_name/:circuit_version", get(ir::get_ir))
        .route("/v1/ir/metadata/list", get(ir::list_irs_metadata))
        .with_state(app_state)
        .fallback(static_files);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(ToSchema)]
#[allow(unused)]
pub struct UnauthorizedResponse {
    message: String,
}
