use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use tracing::{error, info, info_span, Instrument};

use crate::Context;

#[derive(Debug, Serialize, Deserialize)]
pub struct SingupData {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

fn valid_username(username: &str) -> bool {
    username
        .chars()
        .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
}

pub async fn signup(
    State(context): State<Arc<Context>>,
    Json(request): Json<SingupData>,
) -> impl IntoResponse {
    let span = info_span!("signup");
    let _guard = span.enter();

    info!(
        event = "Handle request",
        event = "Request to signup user",
        username = request.username,
    );

    if !valid_username(&request.username) {
        error!(
            event = "Username validation failed",
            username = request.username,
        );

        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Unacceptable username"
            })),
        );
    }

    const INSERT_QUERY: &str = "INSERT INTO users (username, password) VALUES ($1, $2);";

    let salt = SaltString::generate(&mut OsRng);

    let Ok(password_hash) = Pbkdf2.hash_password(request.password.as_bytes(), &salt) else {
        error!(event = "Bad password", password = request.password,);

        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Unacceptable password"
            })),
        );
    };

    if sqlx::query(INSERT_QUERY)
        .bind(&request.username)
        .bind(password_hash.to_string())
        .execute(&context.database_connection)
        .await
        .is_err()
    {
        error!(
            event = "Failed to save user into database",
            username = request.username,
        );

        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "User already exists"
            })),
        );
    }

    info!(event = "User saved", username = request.username,);

    (StatusCode::CREATED, Json(serde_json::json!({})))
}

pub async fn login(
    State(context): State<Arc<Context>>,
    Json(request): Json<LoginData>,
) -> impl IntoResponse {
    let span = info_span!("signup");
    let _guard = span.enter();

    info!(event = "Request to login user", username = request.username,);

    const LOGIN_QUERY: &str = "SELECT id, password FROM users WHERE users.username = $1;";

    let Some((_, password)): Option<(Uuid, String)> = sqlx::query_as(LOGIN_QUERY)
        .bind(&request.username)
        .fetch_optional(&context.database_connection)
        .in_current_span()
        .await
        .unwrap()
    else {
        error!(event = "Unknown user", username = request.username,);

        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Unknown user"
            })),
        );
    };

    let parsed_hash = PasswordHash::new(&password).unwrap();

    if Pbkdf2
        .verify_password(request.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        error!(event = "Wrong password",);

        return (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "error": "Wrong password"
            })),
        );
    }

    info!(event = "Successfully login", username = request.username,);

    (StatusCode::OK, Json(serde_json::json!({})))
}
