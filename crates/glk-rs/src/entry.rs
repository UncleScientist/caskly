use crate::gestalt::OutputType;
use crate::gestalt::*;
use crate::keycode::Keycode;
use crate::stream::{GlkStreamID, StreamManager};
use crate::windows::{
    GlkWindow, GlkWindowSize, GlkWindowType, WindowManager, WindowRef, WindowSplitMethod,
    WindowType,
};
use crate::GlkRock;

/// The GLK object. TODO: Insert basic usage here
/// This is the API for GLK interpreted as a Rust API.
#[derive(Default, Debug)]
pub struct Glk<T: GlkWindow + Default + 'static> {
    windows: Vec<WindowRef<T>>,
    winmgr: WindowManager<T>,
    stream_mgr: StreamManager,
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
            let (pair, win) = parent.split(method, wintype, rock);
            self.windows.push(pair);
            win
        } else {
            assert!(
                self.windows.is_empty(),
                "new windows must be split from existing ones"
            );
            self.winmgr.open_window(wintype, rock)
        };

        let stream_id = self.stream_mgr.new_stream(new_win.get_winref());
        new_win.set_stream_id(stream_id);

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

    /*
     * Section 3.7 - Other Window Functions
     */

    /// iterate through all the windows
    pub fn window_iterate(&self) -> std::slice::Iter<WindowRef<T>> {
        // should we be doing this with Iter<&WindowRef<T>> instead?
        self.windows.iter()
    }

    /// get the rock value for a given window
    pub fn window_get_rock(&self, win: &WindowRef<T>) -> GlkRock {
        win.get_rock()
    }

    /// get the type of the window
    pub fn window_get_type(&self, win: &WindowRef<T>) -> GlkWindowType {
        win.get_type()
    }

    /// get the parent for this window
    pub fn window_get_parent(&self, win: &WindowRef<T>) -> Option<WindowRef<T>> {
        let parent = win.get_parent()?;
        if parent.winref.borrow().wintype == WindowType::Root {
            None
        } else {
            Some(parent)
        }
    }

    /// get the sibling of this window
    pub fn window_get_sibling(&self, win: &WindowRef<T>) -> Option<WindowRef<T>> {
        win.get_sibling()
    }

    /// gets the root window - if there are no windows, returns None
    pub fn window_get_root(&self) -> Option<WindowRef<T>> {
        self.winmgr.get_root()
    }

    /// clears the window
    pub fn window_clear(&self, win: &WindowRef<T>) {
        win.clear()
    }

    /// get the stream associated with a window
    pub fn window_get_stream(&self, win: &WindowRef<T>) -> GlkStreamID {
        win.get_stream()
    }

    /// write a byte to a stream
    pub fn put_char(&self, streamid: GlkStreamID, ch: u8) {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            println!("glk - put char");
            stream.put_char(ch);
        }
    }
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

    #[test]
    fn can_iterate_windows() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let win2 = glk
            .window_open(
                Some(&win1),
                GlkWindowType::TextGrid,
                Some(WindowSplitMethod {
                    position: WindowSplitPosition::Above,
                    amount: WindowSplitAmount::Proportional(40),
                    border: false,
                }),
                84,
            )
            .unwrap();
        let win3 = glk
            .window_open(
                Some(&win2),
                GlkWindowType::TextGrid,
                Some(WindowSplitMethod {
                    position: WindowSplitPosition::Below,
                    amount: WindowSplitAmount::Fixed(3),
                    border: false,
                }),
                95,
            )
            .unwrap();

        // pair1, pair2, win1, win2, win3
        let mut found = [false, false, false, false, false];
        let mut i = glk.window_iterate();
        let mut count = 0;
        let mut found_pair = None;
        while let Some(win) = i.next() {
            count += 1;
            if win.is_ref(&win1) {
                found[2] = true;
            } else if win.is_ref(&win2) {
                found[3] = true;
            } else if win.is_ref(&win3) {
                found[4] = true;
            } else if found_pair.is_none() {
                found_pair = Some(win.make_clone());
                found[0] = true;
            } else if let Some(ref f) = found_pair {
                if !f.is_ref(&win) {
                    found[1] = true;
                }
            }
        }
        assert_eq!(count, 5);
        assert_eq!([true, true, true, true, true], found);
    }

    #[test]
    fn can_get_parent_of_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        assert!(glk.window_get_parent(&win1).is_none());
        let win2 = glk
            .window_open(
                Some(&win1),
                GlkWindowType::TextGrid,
                Some(WindowSplitMethod {
                    position: WindowSplitPosition::Above,
                    amount: WindowSplitAmount::Proportional(40),
                    border: false,
                }),
                84,
            )
            .unwrap();
        let parent1 = glk.window_get_parent(&win2).unwrap();
        assert_eq!(parent1.get_type(), GlkWindowType::Pair);
    }

    #[test]
    fn can_get_sibling_of_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        assert!(glk.window_get_sibling(&win1).is_none());

        let win2 = glk
            .window_open(
                Some(&win1),
                GlkWindowType::TextGrid,
                Some(WindowSplitMethod {
                    position: WindowSplitPosition::Above,
                    amount: WindowSplitAmount::Proportional(40),
                    border: false,
                }),
                84,
            )
            .unwrap();
        let sibling = glk.window_get_sibling(&win2).unwrap();
        assert!(sibling.is_ref(&win1));
    }

    #[test]
    fn can_get_root_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        assert!(glk.window_get_root().is_none());
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        assert!(glk.window_get_root().unwrap().is_ref(&win1));
    }

    /*
    #[test]
    fn can_put_byte_style_char_into_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(&win);
        println!("entry - test - put char {stream}");
        glk.put_char(stream, b'x');
        assert_eq!(win.winref.borrow().window.textdata, "x");
    }
    */
}
