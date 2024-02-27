use std::env;

use axum::{
    debug_handler,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::{headers::Host, TypedHeader};
use once_cell::sync::Lazy;

use crate::app_error::AppError;

pub static USER_POOL_DOMAIN: Lazy<Option<String>> = Lazy::new(|| env::var("user_pool_domain").ok());
pub static USER_POOL_CLIENT_ID: Lazy<Option<String>> =
    Lazy::new(|| env::var("user_pool_client_id").ok());
pub static AWS_REGION: Lazy<Option<String>> = Lazy::new(|| env::var("aws_region").ok());

/// Open Google Oauth2
#[utoipa::path(get,
    tag="Auth",
    path="/auth/google",
    responses(
        (status = 307, description = "Redirected"),
    )
)]
#[debug_handler]
pub async fn auth_google(TypedHeader(host): TypedHeader<Host>) -> impl IntoResponse {
    let Some(user_pool_domain) = USER_POOL_DOMAIN.as_ref() else {
        return Err(AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "User pool domain is not set".to_string(),
        ));
    };

    let Some(user_pool_client_id) = USER_POOL_CLIENT_ID.as_ref() else {
        return Err(AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "User pool client id is not set".to_string(),
        ));
    };

    let Some(aws_region) = AWS_REGION.as_ref() else {
        return Err(AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "AWS region is not set".to_string(),
        ));
    };

    let scheme = if host.hostname() == "localhost" {
        "http"
    } else {
        "https"
    };

    Ok(Redirect::temporary(&format!("https://{user_pool_domain}.auth.{aws_region}.amazoncognito.com/oauth2/authorize?client_id={user_pool_client_id}&response_type=code&redirect_uri={scheme}://{host}/auth/callback&identity_provider=Google")))
}
