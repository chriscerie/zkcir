use aws_sdk_iam::{
    types::{error::NoSuchEntityException, User},
    Client,
};
use axum::http::StatusCode;

use crate::app_error::AppError;

pub async fn upsert_iam_user(iam_client: &Client, jwt_sub: &String) -> Result<User, AppError> {
    if let Err(error) = iam_client.get_user().user_name(jwt_sub).send().await {
        match &error.into_service_error() {
            aws_sdk_iam::operation::get_user::GetUserError::NoSuchEntityException(
                NoSuchEntityException { .. },
            ) => {
                iam_client.create_user().user_name(jwt_sub).send().await?;

                iam_client
                    .attach_user_policy()
                    .user_name(jwt_sub)
                    .policy_arn("arn:aws:iam::aws:policy/AWSCodeCommitPowerUser")
                    .send()
                    .await?;
            }
            e => {
                tracing::error!("Could not get iam user info: {:?}", e);

                return Err(AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Could not get iam user info".to_string(),
                ));
            }
        }
    }

    let Some(user) = iam_client.get_user().user_name(jwt_sub).send().await?.user else {
        return Err(AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not get iam user info".to_string(),
        ));
    };

    Ok(user)
}
