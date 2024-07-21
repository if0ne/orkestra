use pbkdf2::{
    password_hash::{PasswordHasher, SaltString},
    Pbkdf2,
};
use rand::rngs::OsRng;

use crate::shared::database::Database;

use super::{dto::SignupData, error::SignupError};

pub async fn signup(database: &Database, data: SignupData) -> Result<(), SignupError> {
    if !valid_username(&data.username) {
        return Err(SignupError::InvalidUsername);
    }

    const INSERT_QUERY: &str = "INSERT INTO users (username, password) VALUES ($1, $2);";

    let salt = SaltString::generate(&mut OsRng);

    let Ok(password_hash) = Pbkdf2.hash_password(data.password.as_bytes(), &salt) else {
        return Err(SignupError::InvalidPassword);
    };

    if sqlx::query(INSERT_QUERY)
        .bind(&data.username)
        .bind(password_hash.to_string())
        .execute(database.as_ref())
        .await
        .is_err()
    {
        return Err(SignupError::AlreadyExists);
    }

    Ok(())
}

fn valid_username(username: &str) -> bool {
    username
        .chars()
        .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
}
