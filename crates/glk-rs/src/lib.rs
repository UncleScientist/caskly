#![warn(missing_docs)]

//! A rust library for writing glk-based code

/// The main entry point for Glk
pub mod entry;

/// The gestalt module
pub mod gestalt;

/// The keycode module
pub mod keycode;

/// The windows module
pub mod windows;

/// The events module
pub mod events;

/// The prelude for the library
pub mod prelude {
    /// A rock value
    pub type GlkRock = u32;

    /// File Modes
    #[derive(Debug, Copy, Clone, PartialEq)]
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

    impl GlkFileMode {
        /// is one of the read modes
        pub fn is_read(&self) -> bool {
            matches!(self, GlkFileMode::Read | GlkFileMode::ReadWrite)
        }

        /// is one of the write modes
        pub fn is_write(&self) -> bool {
            matches!(
                self,
                GlkFileMode::Write | GlkFileMode::ReadWrite | GlkFileMode::WriteAppend
            )
        }
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
    pub use crate::entry::Glk;
    pub use crate::events::*;
    pub use crate::gestalt::*;
    pub use crate::keycode::*;
    pub use crate::windows::*;
}

use prelude::*;

pub(crate) mod file_stream;
pub(crate) mod mem_stream;
pub(crate) mod stream;
