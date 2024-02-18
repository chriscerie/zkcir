use aws_sdk_iam::types::StatusType;
use axum::{
    debug_handler,
    extract::{Path, State},
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{app_error::AppError, iam::upsert_iam_user, jwt, state::AppState};

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct CreateKeyPayload {
    /// RSA SSH public key
    key: String,
}

/// Create SSH key
#[utoipa::path(put,
    tag="IR",
    path="/v1/ssh",
    responses(
        (status = 201, description = "Created key", body = String),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn create_key(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Json(payload): Json<CreateKeyPayload>,
) -> Result<impl IntoResponse, AppError> {
    let token = bearer
        .token()
        .to_string()
        .trim_start_matches("Bearer ")
        .to_string();
    let user_data = jwt::get_user_claims(&token)?;

    let iam_client = app_state.get_iam_client();

    let iam_user = upsert_iam_user(iam_client, &user_data.claims.sub).await?;

    iam_client
        .upload_ssh_public_key()
        .user_name(iam_user.user_name)
        .ssh_public_key_body(payload.key)
        .send()
        .await?;

    Ok("Created key".to_string())
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Key {
    /// Public identifier of key
    id: String,

    fingerprint: String,

    /// ISO 8601 date-time format
    upload_time: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ListKeysResponse {
    keys: Vec<Key>,
}

/// List SSH keys
#[utoipa::path(get,
    tag="IR",
    path="/v1/ssh",
    responses(
        (status = 200, description = "Get keys", body = Vec<String>),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn list_keys(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let token = bearer
        .token()
        .to_string()
        .trim_start_matches("Bearer ")
        .to_string();
    let user_data = jwt::get_user_claims(&token)?;

    let iam_client = app_state.get_iam_client();

    let iam_user = upsert_iam_user(iam_client, &user_data.claims.sub).await?;

    let keys = iam_client
        .list_ssh_public_keys()
        .user_name(&iam_user.user_name)
        .send()
        .await?
        .ssh_public_keys
        .unwrap_or_default()
        .into_iter()
        .filter_map(|key| {
            if key.status != StatusType::Active {
                return None;
            }

            Some(Key {
                id: key.ssh_public_key_id,
                fingerprint: "unknown".to_string(),
                upload_time: key.upload_date.to_string(),
            })
        })
        .collect::<Vec<_>>();

    Response::builder()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&ListKeysResponse { keys }).map_err(AppError::from)?)
        .map_err(AppError::from)
}

/// Delete key
#[utoipa::path(delete,
    tag="IR",
    path="/v1/ssh/:key_id",
    responses(
        (status = 204, description = "Deleted key", body = String),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn delete_key(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Path(key_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let token = bearer
        .token()
        .to_string()
        .trim_start_matches("Bearer ")
        .to_string();
    let user_data = jwt::get_user_claims(&token)?;

    let iam_client = app_state.get_iam_client();

    let iam_user = upsert_iam_user(iam_client, &user_data.claims.sub).await?;

    iam_client
        .delete_ssh_public_key()
        .user_name(iam_user.user_name)
        .ssh_public_key_id(key_id)
        .send()
        .await?;

    Ok("Deleted key".to_string())
}
