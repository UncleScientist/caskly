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

    /// Can a given character be printed by the glk library?
    CharOutput(Keycode),
}

/// The responses for different gestalt queries
#[derive(PartialEq, Debug)]
pub enum GestaltResult {
    /// The version number of the current library
    Version(u32),

    /// Is the requested gestalt entry handled by the Glk library?
    Accepted(bool),

    /// Method by which a character can be printed
    CharOutput(OutputType),
}

/// The way a given character will be represented on screen
#[derive(PartialEq, Debug)]
pub enum OutputType {
    /// This character cannot be printed, and may not display anything
    CannotPrint(u32),

    /// The character will be displayed exactly as requested
    ExactPrint,

    /// The character will be approximated using possibly multiple characters
    ApproxPrint(u32),
}
