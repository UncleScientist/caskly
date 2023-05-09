use crate::{keycode::Keycode, windows::WindowType};

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

    /// Can the glk library handle mouse input
    MouseInput,

    /// Does the glk library support timers
    Timer,

    /// Can the glk library display graphics
    Graphics,

    /// Can the glk's library graphics routines handle transparency?
    GraphicsTransparency,

    /// Can the glk library return character input events from graphics windows
    GraphicsCharInput,

    /// Can the glk library draw images in a window of a given type
    DrawImage(WindowType),

    /// Can we handle unicode
    Unicode,

    /// Can the library do unicode normalization
    UnicodeNorm,

    /// Can play sound with the pre-0.7.3 library functions
    Sound,

    /// Can set sound volume
    SoundVolume,

    /// Can send events when sound stops playing
    SoundNotify,

    /// Can the library handle "MOD" style resource sounds
    SoundMusic,

    /// Can the library handle 0.7.3 and later sound library functions
    Sound2,

    /// Can the library suppress the line input being echoed to the window
    LineInputEcho,

    /// Can the VM tell the library what line terminator characters there are
    LineTerminators,

    /// Is the specified character able to be used as a line terminator
    LineTerminatorKey(Keycode),

    /// Can we retrieve the date and time from the glk library
    DateTime,

    /// Can we open and read resources streams
    ResourceStream,
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
