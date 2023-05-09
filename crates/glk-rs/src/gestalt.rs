use crate::keycode::Keycode;

/// The gestalt system
#[derive(PartialEq, Debug)]
pub enum Gestalt {
    /// retrieve the version value
    Version,

    /// Can LineInput accept a given Latin-1 character
    LineInput(char),

    /// Can CharInput accept a given Latin-1 character
    CharInput(Keycode),
}

/// The responses for different gestalt queries
#[derive(PartialEq, Debug)]
pub enum GestaltResult {
    /// The version number of the current library
    Version(u32),

    /// Is the requested gestalt entry handled by the Glk library?
    Accepted(bool),
}
