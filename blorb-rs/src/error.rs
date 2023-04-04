use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum BlorbError {
    #[error("Invalid resource type {0}")]
    InvalidResourceType(String),
}
