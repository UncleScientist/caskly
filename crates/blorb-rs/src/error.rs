use thiserror::Error;

/// Error module
#[derive(Error, Debug, PartialEq)]
pub enum BlorbError {
    /// Input file is not of an IFF "FORM" type
    #[error("Not a FORM file")]
    InvalidFileType,

    /// Attempted to read a resource type that does not exist in blorb files
    #[error("Invalid resource type {0}")]
    InvalidResourceType(String),

    /// User asked for an invalid resource ID
    #[error("No such resource {0}")]
    NonExistentResource(usize),
}
