use crate::gestalt::OutputType;
use crate::gestalt::*;
use crate::keycode::Keycode;
use crate::windows::{
    GlkWindow, GlkWindowSize, GlkWindowType, WindowManager, WindowRef, WindowSplitMethod,
    WindowType,
};
use crate::GlkRock;

/// The GLK object. TODO: Insert basic usage here
/// This is the API for GLK interpreted as a Rust API.
#[derive(Default, Debug)]
pub struct Glk<T: GlkWindow + Default> {
    windows: Vec<WindowRef<T>>,
    winmgr: WindowManager<T>,
}

trait ValidGlkChar {
    fn is_glk_char(&self) -> bool;
}

impl ValidGlkChar for char {
    // The Glk spec requires that all valid characters are in the range 32 to 126.
    fn is_glk_char(&self) -> bool {
        (32..=126).contains(&(*self as i32))
    }
}

impl<T: GlkWindow + Default> Glk<T> {
    /// Create a new glk interface
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve capability from the gestalt system
    pub fn gestalt(&self, gestalt: Gestalt) -> GestaltResult {
        match gestalt {
            Gestalt::Version => GestaltResult::Version(0x00000705),
            Gestalt::LineInput(ch) => GestaltResult::CanAccept(ch.is_glk_char()),
            Gestalt::CharInput(Keycode::Basic(ch)) => GestaltResult::CanAccept(ch.is_glk_char()),
            Gestalt::CharInput(ch) => GestaltResult::CanAccept(Keycode::Return == ch),
            Gestalt::CharOutput(Keycode::Basic(ch)) => {
                if ch.is_glk_char() {
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
    pub fn char_to_lower(&self, ch: impl ToChar) -> char {
        let ch = ch.to_char();
        ch.to_lowercase().next().unwrap()
    }

    /// Convert a latin-1 / unicode character to uppercase
    pub fn char_to_upper(&self, ch: impl ToChar) -> char {
        let ch = ch.to_char();
        ch.to_uppercase().next().unwrap()
    }

    /// convert a string to upper case
    pub fn buffer_to_upper_case_uni(&self, s: &str) -> String {
        s.to_uppercase()
    }

    /// convert a string to lower case
    pub fn buffer_to_lower_case_uni(&self, s: &str) -> String {
        s.to_lowercase()
    }

    /// convert a string to title case
    pub fn buffer_to_title_case_uni(&self, s: &str, style: TitleCaseStyle) -> String {
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

    /*
     * Glk Section 3.2 - Window Opening, Closing, and Constraints
     */
    /// create a new window
    pub fn window_open(
        &mut self,
        parent: Option<&WindowRef<T>>,
        wintype: GlkWindowType,
        method: Option<WindowSplitMethod>,
        rock: GlkRock,
    ) -> Option<WindowRef<T>> {
        let wintype = match wintype {
            GlkWindowType::Blank => WindowType::Blank,
            GlkWindowType::TextBuffer => WindowType::TextBuffer,
            GlkWindowType::TextGrid => WindowType::TextGrid,
            GlkWindowType::Graphics => WindowType::Graphics,
            GlkWindowType::Pair => return None,
        };

        let new_win = if let Some(parent) = parent {
            parent.split(method, wintype, rock)
        } else {
            assert!(
                self.windows.is_empty(),
                "new windows must be split from existing ones"
            );
            self.winmgr.open_window(wintype, rock)
        };

        self.windows.push(new_win);
        Some(self.windows.last()?.make_clone())
    }

    /// close the given window and all of its children
    pub fn window_close(&mut self, win: &WindowRef<T>) {
        self.windows.retain(|w| !w.is_ref(win));
        win.close_window();
    }

    /*
     * Glk Spec section 3.3 - Changing Window Constraints
     */

    /// get the actual size of the window, in its measurement system
    pub fn window_get_size(&self, win: &WindowRef<T>) -> GlkWindowSize {
        win.get_size()
    }

    /// Get the size of the window in its measurement system (Glk Spec section 1.9)
    pub fn window_set_arrangement(
        &self,
        win: &WindowRef<T>,
        method: WindowSplitMethod,
        keywin: Option<&WindowRef<T>>,
    ) {
        win.set_arrangement(method, keywin);
    }

    /// returns the constraints of the window
    pub fn window_get_arrangement(
        &self,
        win: &WindowRef<T>,
    ) -> (Option<WindowSplitMethod>, Option<WindowRef<T>>) {
        if let Some((method, keywin)) = win.get_arrangement() {
            (Some(method), keywin)
        } else {
            (None, None)
        }
    }

    /*
     * Section 3.5.4 - Text Grid Windows
     */

    /// Move the cursor in a text grid window (all other window types ignore this API)
    pub fn window_move_cursor(&self, win: &WindowRef<T>, xpos: u32, ypos: u32) {
        win.move_cursor(xpos, ypos);
    }

    /// get the rock value for a given window
    pub fn window_get_rock(&self, win: &WindowRef<T>) -> GlkRock {
        win.get_rock()
    }

    /// get the type of the window
    pub fn window_get_type(&self, win: &WindowRef<T>) -> GlkWindowType {
        win.get_type()
    }

    /*
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
    use crate::windows::{testwin::GlkTestWindow, WindowSplitAmount, WindowSplitPosition};

    #[test]
    fn can_get_glk_version() {
        let glk = Glk::<GlkTestWindow>::new();
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
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!(
            GestaltResult::CanAccept(true),
            glk.gestalt(Gestalt::CharInput(Keycode::Basic('a')))
        );
    }

    #[test]
    fn can_handle_return_key() {
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!(
            GestaltResult::CanAccept(true),
            glk.gestalt(Gestalt::CharInput(Keycode::Return))
        );
    }

    #[test]
    fn can_output_normal_characters() {
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!(
            GestaltResult::CharOutput(OutputType::ExactPrint),
            glk.gestalt(Gestalt::CharOutput(Keycode::Basic('f')))
        );
    }

    #[test]
    fn cannot_print_invalid_characters() {
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!(
            GestaltResult::CharOutput(OutputType::CannotPrint(1)),
            glk.gestalt(Gestalt::CharOutput(Keycode::Basic('\t')))
        );
    }

    #[test]
    fn can_convert_to_uppercase() {
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!('A', glk.char_to_upper('a'));
    }

    #[test]
    fn can_convert_to_lowercase() {
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!('a', glk.char_to_lower('A'));
    }

    #[test]
    fn can_do_non_english_chars() {
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!('ü', glk.char_to_lower('Ü'));
    }

    #[test]
    fn convert_string_to_uppercase() {
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!("ABCDEF".to_string(), glk.buffer_to_upper_case_uni("AbcDef"));
    }

    #[test]
    fn convert_string_to_lowercase() {
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!("abcdef".to_string(), glk.buffer_to_lower_case_uni("AbcDef"));
    }

    #[test]
    fn convert_string_to_title_case() {
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!(
            "AbcDef",
            glk.buffer_to_title_case_uni("abcDef", TitleCaseStyle::UppercaseFirst)
        );
    }

    #[test]
    fn convert_string_to_title_case_with_lowercase() {
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!(
            "Abcdef",
            glk.buffer_to_title_case_uni("abcDef", TitleCaseStyle::LowercaseRest)
        );
    }

    #[test]
    fn conversion_of_title_case_handles_empty_string() {
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!(
            "",
            glk.buffer_to_title_case_uni("", TitleCaseStyle::LowercaseRest)
        );
    }

    #[test]
    fn can_create_a_window() {
        let mut glk = Glk::<GlkTestWindow>::new();

        let win = glk.window_open(None, GlkWindowType::TextBuffer, None, 73);
        assert!(win.is_some());
    }

    #[test]
    #[should_panic]
    fn must_use_existing_window_for_splits() {
        let mut glk = Glk::<GlkTestWindow>::new();

        glk.window_open(None, GlkWindowType::TextBuffer, None, 73);
        glk.window_open(None, GlkWindowType::TextBuffer, None, 73);
    }

    #[test]
    fn can_create_a_split_window() {
        let mut glk = Glk::<GlkTestWindow>::new();

        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let win2 = glk.window_open(
            Some(&win),
            GlkWindowType::TextGrid,
            Some(WindowSplitMethod {
                position: WindowSplitPosition::Above,
                amount: WindowSplitAmount::Proportional(40),
                border: false,
            }),
            84,
        );
        assert!(win2.is_some());
    }

    #[test]
    fn can_retrieve_window_information() {
        let mut glk = Glk::<GlkTestWindow>::new();

        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        assert_eq!(win.get_rock(), 73);
        assert_eq!(glk.window_get_rock(&win), 73);
        assert_eq!(glk.window_get_type(&win), GlkWindowType::TextBuffer);
    }
}
