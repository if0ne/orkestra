use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SignupError {
    #[error("Unknown user")]
    InvalidUsername,

    #[error("Wrong password")]
    InvalidPassword,

    #[error("User already exists")]
    AlreadyExists,
}
