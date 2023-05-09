#![warn(missing_docs)]

//! A rust library for writing glk-based code

/// The main entry point for all things glk
pub mod entry;
pub use entry::Glk;

/// The gestalt subsystem
pub mod gestalt;
