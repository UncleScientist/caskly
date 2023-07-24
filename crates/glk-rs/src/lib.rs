#![warn(missing_docs)]

//! A rust library for writing glk-based code

/// A rock value
type GlkRock = u32;

/// File Modes
pub enum GlkFileMode {
    /// Stream is read-only
    Read,

    /// Stream is write-only
    Write,

    /// Stream is read-write
    ReadWrite,
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
