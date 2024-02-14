use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub struct AppError {
    inner: anyhow::Error,
    status_code: StatusCode,
    message: String,
}

impl AppError {
    #[allow(dead_code)]
    pub fn new(status_code: StatusCode, message: String) -> Self {
        Self {
            inner: anyhow::anyhow!(message),
            status_code,
            message: "Encountered error".to_string(),
        }
    }

    pub fn new_error<E>(status_code: StatusCode, err: E) -> Self
    where
        E: Into<anyhow::Error>,
    {
        Self {
            inner: err.into(),
            status_code,
            message: "Encountered error".to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            self.status_code,
            format!("{}: {}", self.message, self.inner),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error> + std::fmt::Debug,
{
    fn from(err: E) -> Self {
        tracing::error!("Encountered internal error: {:?}", err);
        Self::new_error(StatusCode::INTERNAL_SERVER_ERROR, err)
    }
}
