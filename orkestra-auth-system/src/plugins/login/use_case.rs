use pbkdf2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Pbkdf2,
};
use sqlx::types::Uuid;
use tracing::Instrument;

use crate::shared::database::Database;

use super::{dto::LoginData, error::LoginError};

pub async fn login(database: &Database, data: LoginData) -> Result<(), LoginError> {
    const LOGIN_QUERY: &str = "SELECT id, password FROM users WHERE users.username = $1;";

    let Some((_, password)): Option<(Uuid, String)> = sqlx::query_as(LOGIN_QUERY)
        .bind(&data.username)
        .fetch_optional(database.as_ref())
        .in_current_span()
        .await
        .unwrap()
    else {
        return Err(LoginError::UnknownUser);
    };

    let parsed_hash = PasswordHash::new(&password).unwrap();

    if Pbkdf2
        .verify_password(data.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(LoginError::WrongPassword);
    }

    Ok(())
}
