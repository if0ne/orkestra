use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LoginError {
    #[error("Unknown user")]
    UnknownUser,
    #[error("Wrong password")]
    WrongPassword,
}
