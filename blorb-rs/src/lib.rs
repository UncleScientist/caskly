#![warn(missing_docs)]

//! A library for reading blorb files.
//!
//! let bytes = std::fs::read("my_game.gblorb").unwrap();
//! let blorb_file = BlorbReader::new(bytes);
//!
//! let gamedata: &[u8] = blorb_file.get_resource_by_id(0);
//! let imagedata: &[u8] = blorb_file.get_resource_by_type(BlorbType::PNG);
//!

/// reader
pub mod reader;

/// chunks
pub mod chunk;

/// errors
pub mod error;

/// types
pub mod types;
