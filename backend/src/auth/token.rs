use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{config::Config, error::{AppError, AppResult}};

#[derive(Clone)]
pub struct TokenService {
    encoding: EncodingKey,
    decoding: DecodingKey,
    access_minutes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub sid: Uuid,
    pub iat: usize,
    pub exp: usize,
    pub iss: String,
}

impl TokenService {
    pub fn new(config: &Config) -> AppResult<Self> {
        Ok(Self {
            encoding: EncodingKey::from_rsa_pem(&config.jwt_private_key)
                .map_err(|e| AppError::Config(format!("chave JWT privada inválida: {e}")))?,
            decoding: DecodingKey::from_rsa_pem(&config.jwt_public_key)
                .map_err(|e| AppError::Config(format!("chave JWT pública inválida: {e}")))?,
            access_minutes: config.access_token_minutes,
        })
    }

    pub fn access_token(&self, user_id: Uuid, session_id: Uuid) -> AppResult<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            sid: session_id,
            iat: now.timestamp() as usize,
            exp: (now + Duration::minutes(self.access_minutes)).timestamp() as usize,
            iss: "ascendia-online".into(),
        };
        encode(&Header::new(Algorithm::RS256), &claims, &self.encoding)
            .map_err(|e| AppError::Internal(format!("jwt encode: {e}")))
    }

    pub fn verify(&self, token: &str) -> AppResult<Claims> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&["ascendia-online"]);
        decode::<Claims>(token, &self.decoding, &validation)
            .map(|data| data.claims)
            .map_err(|_| AppError::Unauthorized)
    }
}

pub fn new_refresh_token() -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(64).map(char::from).collect()
}

pub fn token_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}
