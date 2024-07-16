use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use tracing::{error, info};

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
) -> Result<StatusCode, StatusCode> {
    info!(
        target: "Signup",
        event = "Request to signup user",
        username = request.username,
    );

    if !valid_username(&request.username) {
        error!(
            target: "Signup",
            event = "Validation failed",
            username = request.username,
        );

        return Err(StatusCode::BAD_REQUEST);
    }

    const INSERT_QUERY: &str = "INSERT INTO users (username, password) VALUES ($1, $2);";

    let salt = SaltString::generate(&mut OsRng);

    let Ok(password_hash) = Pbkdf2.hash_password(request.password.as_bytes(), &salt) else {
        error!(
            target: "Signup",
            event = "Bad password",
            password = request.password,
        );

        return Err(StatusCode::BAD_REQUEST);
    };

    if sqlx::query(INSERT_QUERY)
        .bind(&request.username)
        .bind(password_hash.to_string())
        .execute(&context.database_connection)
        .await
        .is_err()
    {
        error!(
            target: "Signup",
            event = "Failed to save user into database",
            username = request.username,
        );

        return Err(StatusCode::BAD_REQUEST);
    }

    info!(
        target: "Signup",
        event = "User saved",
        username = request.username,
    );

    Ok(StatusCode::CREATED)
}

pub async fn login(
    State(context): State<Arc<Context>>,
    Json(request): Json<LoginData>,
) -> Result<StatusCode, StatusCode> {
    info!(
        target: "Login",
        event = "Request to login user",
        username = request.username,
    );

    const LOGIN_QUERY: &str = "SELECT id, password FROM users WHERE users.username = $1;";

    let Some((_, password)): Option<(Uuid, String)> = sqlx::query_as(LOGIN_QUERY)
        .bind(&request.username)
        .fetch_optional(&context.database_connection)
        .await
        .unwrap()
    else {
        error!(
            target: "Login",
            event = "Unknown username",
            username = request.username,
        );

        return Err(StatusCode::BAD_REQUEST);
    };

    let parsed_hash = PasswordHash::new(&password).unwrap();

    if Pbkdf2
        .verify_password(request.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        error!(
            target: "Login",
            event = "Wrong password",
        );

        return Err(StatusCode::BAD_REQUEST);
    }

    info!(
        target: "Login",
        event = "Successfully login",
        username = request.username,
    );

    Ok(StatusCode::OK)
}
