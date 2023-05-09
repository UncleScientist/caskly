use crate::gestalt::*;
use crate::keycode::Keycode;

/// The GLK object. TODO: Insert basic usage here
#[derive(Default)]
pub struct Glk;

impl Glk {
    /// Create a new glk interface
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve capability from the gestalt system
    pub fn gestalt(&self, gestalt: Gestalt) -> GestaltResult {
        match gestalt {
            Gestalt::Version => GestaltResult::Version(0x00000705),
            Gestalt::LineInput(ch) => GestaltResult::Accepted(ch as u32 >= 32 && (ch as u32) < 127),
            Gestalt::CharInput(Keycode::Basic(ch)) => {
                GestaltResult::Accepted(ch as u32 >= 32 && (ch as u32) < 127)
            }
            Gestalt::CharInput(ch) => GestaltResult::Accepted(Keycode::Return == ch),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_get_glk_version() {
        let glk = Glk::new();
        assert_eq!(
            GestaltResult::Version(0x00000705),
            glk.gestalt(Gestalt::Version)
        );
    }

    #[test]
    fn can_convert_char_to_keycode() {
        assert_eq!(Keycode::Basic('c'), 'c'.into());
    }
    #[test]
    fn can_handle_characters() {
        let glk = Glk::new();
        assert_eq!(
            GestaltResult::Accepted(true),
            glk.gestalt(Gestalt::CharInput(Keycode::Basic('a')))
        );
    }

    #[test]
    fn can_handle_return_key() {
        let glk = Glk::new();
        assert_eq!(
            GestaltResult::Accepted(true),
            glk.gestalt(Gestalt::CharInput(Keycode::Return))
        );
    }
}
