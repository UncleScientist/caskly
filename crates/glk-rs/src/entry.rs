use crate::gestalt::OutputType;
use crate::gestalt::*;
use crate::keycode::Keycode;
// use crate::windows::{BorderStyle, SplitDirection, SplitMethod, WinID, WindowType};
use crate::windows::{WindowRef, WindowSplitMethod, WindowType};

/// The GLK object. TODO: Insert basic usage here
#[derive(Default)]
pub struct Glk {
    windows: Vec<WindowRef>,
}

impl Glk {
    /// Create a new glk interface
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve capability from the gestalt system
    pub fn gestalt(&self, gestalt: Gestalt) -> GestaltResult {
        match gestalt {
            Gestalt::Version => GestaltResult::Version(0x00000705),
            Gestalt::LineInput(ch) => GestaltResult::CanAccept(ch >= ' ' && ch <= '~'),
            Gestalt::CharInput(Keycode::Basic(ch)) => {
                GestaltResult::CanAccept(ch >= ' ' && ch <= '~')
            }
            Gestalt::CharInput(ch) => GestaltResult::CanAccept(Keycode::Return == ch),
            Gestalt::CharOutput(Keycode::Basic(ch)) => {
                if ch >= ' ' && ch <= '~' {
                    GestaltResult::CharOutput(OutputType::ExactPrint)
                } else {
                    GestaltResult::CharOutput(OutputType::CannotPrint(1))
                }
            }
            Gestalt::CharOutput(_) => GestaltResult::CharOutput(OutputType::CannotPrint(1)),
            Gestalt::Unicode | Gestalt::UnicodeNorm => GestaltResult::CanAccept(true),
            _ => GestaltResult::CanAccept(false),
        }
    }

    /// Convert a latin-1 / unicode character to lowercase
    pub fn char_to_lower(ch: impl ToChar) -> char {
        let ch = ch.to_char();
        ch.to_lowercase().next().unwrap()
    }

    /// Convert a latin-1 / unicode character to uppercase
    pub fn char_to_upper(ch: impl ToChar) -> char {
        let ch = ch.to_char();
        ch.to_uppercase().next().unwrap()
    }

    /// convert a string to upper case
    pub fn buffer_to_upper_case_uni(s: &str) -> String {
        s.to_uppercase()
    }

    /// convert a string to lower case
    pub fn buffer_to_lower_case_uni(s: &str) -> String {
        s.to_lowercase()
    }

    /// convert a string to title case
    pub fn buffer_to_title_case_uni(s: &str, style: TitleCaseStyle) -> String {
        let mut result = String::new();

        if s.is_empty() {
            return result;
        }

        let mut iter = s.chars();

        let first_char = iter.next().unwrap();
        result.push(first_char.to_uppercase().next().unwrap());

        if style == TitleCaseStyle::UppercaseFirst {
            result.extend(iter);
        } else {
            result.extend(iter.map(|x| x.to_lowercase().next().unwrap()));
        }

        result
    }

    /// create a new window
    pub fn window_open(
        &mut self,
        parent: Option<WindowRef>,
        wintype: WindowType,
        method: WindowSplitMethod,
        rock: crate::GlkRock,
    ) -> Option<&WindowRef> {
        // Ideally this would work similar to this:
        // if let Some(new_win) = parent.split(wintype, method, rock) {
        //      self.windows.push(new_win)
        //      self.windows.last()
        // } else {
        //      None
        // }
        if !self.windows.is_empty() || parent.is_some() {
            return None;
        }

        if wintype != WindowType::TextBuffer {
            return None;
        }

        let new_win = WindowRef::init();
        self.windows.push(new_win);

        self.windows.last()
    }

    /*
    /// close the given window and all of its children
    pub fn window_close(&mut self, _win: WinID) {
        todo!();
    }

    /// get the rock value for a given window
    pub fn window_get_rock(win: WinID) -> u32 {
        win.get_rock()
    }

    /// get the parent for this window
    pub fn window_get_parent(win: WinID) -> Option<WinID> {
        win.parent()
    }

    /// get the sibling of this window
    pub fn window_get_sibling(win: WinID) -> Option<WinID> {
        win.sibling()
    }
    */
}

/// determines the style of title case conversions
#[derive(Debug, PartialEq)]
pub enum TitleCaseStyle {
    /// Convert the first character to uppercase, and leave the remaining characters alone
    UppercaseFirst,

    /// Convert the first character to uppercase, and convert the remaining to lowercase
    LowercaseRest,
}

/// Provide a conversion function for u8 (Latin-1) values, and char (unicode) values
pub trait ToChar {
    /// convert value to char
    fn to_char(&self) -> char;
}

impl ToChar for u8 {
    fn to_char(&self) -> char {
        *self as char
    }
}

impl ToChar for char {
    fn to_char(&self) -> char {
        *self
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
            GestaltResult::CanAccept(true),
            glk.gestalt(Gestalt::CharInput(Keycode::Basic('a')))
        );
    }

    #[test]
    fn can_handle_return_key() {
        let glk = Glk::new();
        assert_eq!(
            GestaltResult::CanAccept(true),
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

    #[test]
    fn can_convert_to_uppercase() {
        assert_eq!('A', Glk::char_to_upper('a'));
    }

    #[test]
    fn can_convert_to_lowercase() {
        assert_eq!('a', Glk::char_to_lower('A'));
    }

    #[test]
    fn can_do_non_english_chars() {
        assert_eq!('ü', Glk::char_to_lower('Ü'));
    }

    #[test]
    fn convert_string_to_uppercase() {
        assert_eq!(
            "ABCDEF".to_string(),
            Glk::buffer_to_upper_case_uni("AbcDef")
        );
    }

    #[test]
    fn convert_string_to_lowercase() {
        assert_eq!(
            "abcdef".to_string(),
            Glk::buffer_to_lower_case_uni("AbcDef")
        );
    }

    #[test]
    fn convert_string_to_title_case() {
        assert_eq!(
            "AbcDef",
            Glk::buffer_to_title_case_uni("abcDef", TitleCaseStyle::UppercaseFirst)
        );
    }

    #[test]
    fn convert_string_to_title_case_with_lowercase() {
        assert_eq!(
            "Abcdef",
            Glk::buffer_to_title_case_uni("abcDef", TitleCaseStyle::LowercaseRest)
        );
    }

    #[test]
    fn conversion_of_title_case_handles_empty_string() {
        assert_eq!(
            "",
            Glk::buffer_to_title_case_uni("", TitleCaseStyle::LowercaseRest)
        );
    }
}
