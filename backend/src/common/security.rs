use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub role: String,
    pub exp: usize,
}

/// Nilai `role` pada JWT milik member. Berfungsi sebagai scope terpisah:
/// extractor staff menolak token dengan role ini, extractor member hanya
/// menerima token dengan role ini (dua dunia tidak saling menembus).
/// Kode role staff `member` dicadangkan dan tidak boleh dipakai.
pub const MEMBER_TOKEN_ROLE: &str = "member";

/// `true` bila klaim berasal dari token member-scope.
pub fn is_member_scope(role: &str) -> bool {
    role == MEMBER_TOKEN_ROLE
}

pub fn hash_password(password: &str) -> AppResult<String> {
    if password.len() < 8 {
        return Err(AppError::Validation(
            "password must contain at least 8 characters".to_string(),
        ));
    }

    let salt_source = hex::encode(Uuid::new_v4().as_bytes());
    let salt = SaltString::from_b64(&salt_source)
        .map_err(|error| AppError::Internal(format!("failed to create password salt: {error}")))?;
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| AppError::Internal(format!("failed to hash password: {error}")))?;

    Ok(hash.to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|error| AppError::Internal(format!("invalid stored password hash: {error}")))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn create_token(user_id: i64, role: &str, secret: &str, ttl_seconds: i64) -> AppResult<String> {
    let expires_at = Utc::now() + Duration::seconds(ttl_seconds);
    let claims = Claims {
        sub: user_id,
        role: role.to_string(),
        exp: expires_at.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|error| AppError::Internal(format!("failed to create token: {error}")))
}

pub fn verify_token(token: &str, secret: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}

pub fn generate_refresh_token() -> String {
    format!(
        "{}{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    )
}

pub fn hash_refresh_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_scope_round_trips_through_token() {
        let secret = "test-secret";
        let member_token =
            create_token(42, MEMBER_TOKEN_ROLE, secret, 900).expect("member token encodes");
        let staff_token = create_token(7, "cashier", secret, 900).expect("staff token encodes");

        let member_claims = verify_token(&member_token, secret).expect("member token verifies");
        let staff_claims = verify_token(&staff_token, secret).expect("staff token verifies");

        assert_eq!(member_claims.sub, 42);
        assert!(is_member_scope(&member_claims.role));
        assert!(!is_member_scope(&staff_claims.role));
    }

    #[test]
    fn staff_role_codes_are_not_member_scope() {
        for role in ["admin", "cashier", "owner", ""] {
            assert!(
                !is_member_scope(role),
                "role {role} must not be member scope"
            );
        }
        assert!(is_member_scope("member"));
    }
}
