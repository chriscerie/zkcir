use axum::{debug_handler, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use utoipa::ToSchema;

use crate::{app_error::AppError, jwt};

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetProfileResponse {
    user_id: String,
    email: Option<String>,
    name: Option<String>,
    picture: Option<String>,
}

/// Get user attributes
#[utoipa::path(get,
    tag="Profile",
    path="/v1/profile",
    responses(
        (status = 200, description = "Get user attributes", body = GetProfileResponse),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn get_profile(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<GetProfileResponse>, AppError> {
    let token = bearer.token().trim_start_matches("Bearer ").to_string();
    let data = jwt::get_user_claims(&token)?;

    let response = GetProfileResponse {
        user_id: data.claims.sub,
        email: data.claims.email,
        name: data.claims.name,
        picture: data.claims.picture,
    };

    Ok(Json(response))
}
