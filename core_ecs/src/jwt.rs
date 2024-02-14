use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

pub fn get_user_claims(token: &str) -> Result<TokenData<Claims>, anyhow::Error> {
    let mut validation = Validation::new(Algorithm::HS256);

    // Cognito already validates the JWT token
    validation.insecure_disable_signature_validation();
    validation.validate_exp = false;
    validation.validate_nbf = false;
    validation.validate_aud = false;

    jsonwebtoken::decode::<Claims>(token, &DecodingKey::from_secret(&[]), &validation).map_err(
        |e| {
            tracing::error!("Failed to get user claims: {}", e);
            anyhow::anyhow!("Failed to get user claims")
        },
    )
}
