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

    /// The requested chunk type was not found in the blorb file
    #[error("Chunk not found")]
    ChunkNotFound,

    /// Reached the end of the file
    #[error("End of file")]
    EndOfFile,

    /// Could not convert generic blorb type into a known chunk type
    #[error("Cannot convert")]
    ConversionFailed,

    /// Could not convert slice of bytes into a valid utf8 string
    #[error("Not a utf8 string")]
    InvalidUtf8String,
}
