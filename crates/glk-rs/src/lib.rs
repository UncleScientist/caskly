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

/// File Usages
#[derive(Debug, Default, Clone)]
pub enum GlkFileUsage {
    /// A file which stores game state.
    SavedGame,

    /// A file which contains a stream of text from the game (often an echo stream
    /// from a window.)
    Transcript,

    /// A file which records player input.
    InputRecord,

    /// Any other kind of file (preferences, statistics, arbitrary data.)
    #[default]
    Data,

    /// The file contents will be stored exactly as they are written, and read back
    /// in the same way. The resulting file may not be viewable on platform-native text
    /// file viewers.
    BinaryMode,

    /// The file contents will be transformed to a platform-native text file as they are written
    /// out. Newlines may be converted to linefeeds or linefeed-plus-carriage-return combinations;
    /// Latin-1 characters may be converted to native character codes. When reading a file in text
    /// mode, native line breaks will be converted back to newline (0x0A) characters, and native
    /// character codes may be converted to Latin-1 or UTF-8.
    TextMode,
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

/// File I/O system
pub mod file_stream;

pub(crate) mod mem_stream;
pub(crate) mod stream;
