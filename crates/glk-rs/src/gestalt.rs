/// The gestalt system
#[derive(PartialEq, Debug)]
pub enum Gestalt {
    /// retrieve the version value
    Version,

    /// can LineInput accept a given Latin-1 character
    LineInput(char),
}

/// The responses for different gestalt queries
#[derive(PartialEq, Debug)]
pub enum GestaltResult {
    /// The version number of the current library
    Version(u32),

    /// Is the character able to be read during line input
    LineInput(bool),
}
