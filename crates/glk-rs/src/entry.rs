use crate::gestalt::OutputType;
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
            Gestalt::CharOutput(Keycode::Basic(ch)) => {
                if (ch as u32) >= 32 && (ch as u32) < 127 {
                    GestaltResult::CharOutput(OutputType::ExactPrint)
                } else {
                    GestaltResult::CharOutput(OutputType::CannotPrint(1))
                }
            }
            Gestalt::CharOutput(_) => GestaltResult::CharOutput(OutputType::CannotPrint(1)),
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

    #[test]
    fn can_output_normal_characters() {
        let glk = Glk::new();
        assert_eq!(
            GestaltResult::CharOutput(OutputType::ExactPrint),
            glk.gestalt(Gestalt::CharOutput(Keycode::Basic('f')))
        );
    }

    #[test]
    fn cannot_print_invalid_characters() {
        let glk = Glk::new();
        assert_eq!(
            GestaltResult::CharOutput(OutputType::CannotPrint(1)),
            glk.gestalt(Gestalt::CharOutput(Keycode::Basic('\t')))
        );
    }
}
