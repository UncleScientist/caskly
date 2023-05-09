/// Keycode translated from the i/o input system into GLK codes
#[derive(Debug, PartialEq)]
pub enum Keycode {
    /// A "normal" letter
    Basic(char),
    /// Arrow left
    Left,
    /// Arrow right
    Right,
    /// Arrow up
    Up,
    /// Arrow down
    Down,
    /// Return/Enter key
    Return,
    /// Delete/Backspace key
    Delete,
    /// Escape key
    Escape,
    /// Tab key
    Tab,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Home
    Home,
    /// End
    End,
    /// Function keycode
    Func1,
    /// Function keycode
    Func2,
    /// Function keycode
    Func3,
    /// Function keycode
    Func4,
    /// Function keycode
    Func5,
    /// Function keycode
    Func6,
    /// Function keycode
    Func7,
    /// Function keycode
    Func8,
    /// Function keycode
    Func9,
    /// Function keycode
    Func10,
    /// Function keycode
    Func11,
    /// Function keycode
    Func12,
    /// The keycode could not be translated into something that Glk knows
    Unknown,
}

impl From<char> for Keycode {
    fn from(ch: char) -> Self {
        if (ch as u32) >= 32 && (ch as u32) < 127 {
            return Keycode::Basic(ch);
        }
        if ch == '\r' || ch == '\n' {
            Keycode::Return
        } else {
            Keycode::Unknown
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_convert_return_and_enter_keys() {
        assert_eq!(Keycode::Return, '\r'.into());
        assert_eq!(Keycode::Return, '\n'.into());
    }
}
