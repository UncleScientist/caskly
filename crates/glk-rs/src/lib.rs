#![warn(missing_docs)]

//! A rust library for writing glk-based code

/// A rock value
type GlkRock = u32;

/// File Modes
#[derive(Debug)]
pub enum GlkFileMode {
    /// Stream is read-only
    Read,

    /// Stream is write-only
    Write,

    /// Stream is read-write
    ReadWrite,

    /// Append to end of stream
    WriteAppend,
}

/// Seek Modes
#[derive(Debug)]
pub enum GlkSeekMode {
    /// Seek to a position offset from the beginning of the file
    Start,

    /// Seek to a position relative to the current offset in the file
    Current,

    /// Seek to a position offset from the end of the file (offset must be 0 or negative)
    End,
}

/// The main entry point for all things glk
pub mod entry;
pub use entry::Glk;

/// The gestalt subsystem
pub mod gestalt;

/// Keycode translation table
pub mod keycode;

/// Windowing subsystem
pub mod windows;

pub(crate) mod mem_stream;
/// Streams
pub(crate) mod stream;
