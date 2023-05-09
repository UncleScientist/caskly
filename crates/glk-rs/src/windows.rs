// TODO: re-factor this code when implementing the windows subsystem

/// Types of windows
#[derive(Debug, PartialEq)]
pub enum WindowType {
    /// a text buffer window -- can stream output, and accept line input
    TextBuffer,

    /// A text grid window -- can draw characters at arbitrary x/y coordinates
    TextGrid,

    /// Can display graphics
    Graphics,

    /// A basic window with no input or output facility
    Blank,
}
