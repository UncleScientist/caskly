#![warn(missing_docs)]

//! A rust library for writing glk-based code

/// A rock value
type GlkRock = i32;

/// The main entry point for all things glk
pub mod entry;
pub use entry::Glk;

/// The gestalt subsystem
pub mod gestalt;

/// Keycode translation table
pub mod keycode;

/// Windowing subsystem
pub mod windows;
