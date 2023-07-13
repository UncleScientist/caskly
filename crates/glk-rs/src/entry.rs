use crate::gestalt::OutputType;
use crate::gestalt::*;
use crate::keycode::Keycode;
use crate::stream::{GlkStreamID, GlkStreamResult, StreamManager};
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
    default_stream: Option<GlkStreamID>,
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
    pub fn window_close(&mut self, win: &WindowRef<T>) -> GlkStreamResult {
        self.windows.retain(|w| !w.is_ref(win));
        let stream = win.get_stream();
        win.close_window();
        self.stream_mgr.close(stream).unwrap()
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

    /*
     * Section 5 - Streams
     */

    /// set the current stream, or None to disable
    pub fn stream_set_current(&mut self, streamid: GlkStreamID) {
        self.default_stream = Some(streamid)
    }

    /// get the current stream, or None if no stream is set
    pub fn stream_get_current(&self) -> Option<GlkStreamID> {
        self.default_stream
    }

    /*
     * Section 5.1. How to Print
     */

    /// write a byte to the default stream
    pub fn put_char(&self, ch: u8) {
        if let Some(stream) = self.default_stream {
            self.put_char_stream(stream, ch);
        }
    }

    /// write a string to the default stream
    pub fn put_string(&self, s: &str) {
        if let Some(stream) = self.default_stream {
            self.put_string_stream(stream, s);
        }
    }

    /// write a string to the default stream
    pub fn put_string_uni(&self, s: &str) {
        self.put_string(s);
    }

    /// write a byte buffer to the default stream
    pub fn put_buffer(&self, buf: &[u8]) {
        if let Some(stream) = self.default_stream {
            self.put_buffer_stream(stream, buf);
        }
    }

    /// write a unicode character to the default stream
    pub fn put_char_uni(&self, ch: char) {
        if let Some(stream) = self.default_stream {
            self.put_char_stream_uni(stream, ch);
        }
    }

    /// write a unicode buffer to the default stream
    pub fn put_buffer_uni(&self, buf: &[char]) {
        if let Some(stream) = self.default_stream {
            self.put_buffer_stream_uni(stream, buf);
        }
    }

    /// write a byte to a stream
    pub fn put_char_stream(&self, streamid: GlkStreamID, ch: u8) {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.put_char(ch);
        }
    }

    /// write a unicode string to a stream
    pub fn put_string_stream(&self, streamid: GlkStreamID, s: &str) {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.put_string(s);
        }
    }

    /// write a unicode string to a stream - same as put_string_stream() in rust because
    /// all strings are unicode in rust
    pub fn put_string_stream_uni(&self, streamid: GlkStreamID, s: &str) {
        self.put_string_stream(streamid, s);
    }

    /// write a buffer of bytes to a stream
    pub fn put_buffer_stream(&self, streamid: GlkStreamID, buf: &[u8]) {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.put_buffer(buf);
        }
    }

    /// write a unicode character to a stream
    pub fn put_char_stream_uni(&self, streamid: GlkStreamID, ch: char) {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.put_char_uni(ch);
        }
    }

    /// write a buffer of unicode characters to a stream
    pub fn put_buffer_stream_uni(&self, streamid: GlkStreamID, buf: &[char]) {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.put_buffer_uni(buf);
        }
    }

    /*
     * Section 5.2 - How to Read
     */

    /// read a byte from a stream. If the stream is output-only, or if there are no
    /// more characters to read, return None.
    pub fn get_char_stream(&self, streamid: GlkStreamID) -> Option<u8> {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.get_char()
        } else {
            None
        }
    }

    /// read a stream of bytes
    pub fn get_buffer_stream(&self, streamid: GlkStreamID, len: Option<usize>) -> Vec<u8> {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.get_buffer(len)
        } else {
            Vec::new()
        }
    }

    /// read a stream of bytes until a newline, or until end-of-stream
    pub fn get_line_stream(&self, streamid: GlkStreamID, len: Option<usize>) -> Vec<u8> {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.get_line(len)
        } else {
            Vec::new()
        }
    }

    /// get a unicode character from a stream. If the stream is output-only, or if there
    /// are no more characters to read, return None
    pub fn get_char_stream_uni(&self, streamid: GlkStreamID) -> Option<char> {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.get_char_uni()
        } else {
            None
        }
    }

    /// read a stream of unicode characters
    pub fn get_buffer_stream_uni(&self, streamid: GlkStreamID, len: Option<usize>) -> Vec<char> {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.get_buffer_uni(len)
        } else {
            Vec::new()
        }
    }

    /// read a stream of unicode characters
    pub fn get_line_stream_uni(&self, streamid: GlkStreamID, len: Option<usize>) -> Vec<char> {
        if let Some(stream) = self.stream_mgr.get(streamid) {
            stream.get_line_uni(len)
        } else {
            Vec::new()
        }
    }

    /*
     * 5.3 - Closing Streams
     */

    /// Closes a stream. Window streams are only close-able through glk.window_close()
    pub fn stream_close(&mut self, streamid: GlkStreamID) -> Option<GlkStreamResult> {
        let stream = self.stream_mgr.get(streamid)?;
        if stream.is_window_stream() {
            None
        } else {
            self.stream_mgr.close(streamid)
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

    #[test]
    fn can_put_byte_style_char_into_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(&win);
        glk.put_char_stream(stream, b'x');
        assert_eq!(win.winref.borrow().window.borrow().textdata, "x");
    }

    #[test]
    fn can_write_to_two_different_windows() {
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

        let stream1 = glk.window_get_stream(&win1);
        let stream2 = glk.window_get_stream(&win2);

        glk.put_char_stream(stream1, b'A');
        glk.put_char_stream(stream2, b'B');

        assert_eq!(win1.winref.borrow().window.borrow().textdata, "A");
        assert_eq!(win2.winref.borrow().window.borrow().textdata, "B");
    }

    #[test]
    fn can_put_string_into_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(&win);
        glk.put_string_stream(stream, &"hello, world!");
        assert_eq!(
            win.winref.borrow().window.borrow().textdata,
            "hello, world!"
        );
    }

    #[test]
    fn can_put_buffer_into_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(&win);
        glk.put_buffer_stream(stream, &[b'0', b'1', b'2', b'3']);
        assert_eq!(win.winref.borrow().window.borrow().textdata, "0123");
    }

    #[test]
    fn can_put_unicode_char_into_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(&win);
        glk.put_char_stream_uni(stream, 'q');
        assert_eq!(win.winref.borrow().window.borrow().textdata, "q");
    }

    #[test]
    fn can_put_unicode_buf_into_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(&win);
        glk.put_buffer_stream_uni(stream, &['q', 'r', 's', 't', 'u', 'v']);
        assert_eq!(win.winref.borrow().window.borrow().textdata, "qrstuv");
    }

    #[test]
    fn at_startup_there_is_no_default_stream() {
        let glk = Glk::<GlkTestWindow>::new();
        assert!(glk.stream_get_current().is_none());
    }

    #[test]
    fn can_change_default_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(&win);
        glk.stream_set_current(stream);
        assert!(glk.stream_get_current().is_some());
        assert_eq!(glk.stream_get_current(), Some(stream));
    }

    #[test]
    fn can_write_to_default_stream() {
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

        let stream1 = glk.window_get_stream(&win1);
        let stream2 = glk.window_get_stream(&win2);

        glk.stream_set_current(stream1);
        glk.put_char(b'A');
        glk.put_string("bove");
        glk.put_buffer(&[b' ', b't', b'h', b'e']);
        glk.put_char_uni(' ');
        glk.put_buffer_uni(&['s', 'k', 'y']);

        glk.stream_set_current(stream2);
        glk.put_char(b'B');
        glk.put_string("elow");
        glk.put_buffer(&[b' ', b'g', b'r', b'o', b'u', b'n', b'd']);
        glk.put_char_uni('.');
        glk.put_buffer_uni(&[' ', 'L', 'o', 'o', 'k', '!']);

        assert_eq!(
            win1.winref.borrow().window.borrow().textdata,
            "Above the sky"
        );
        assert_eq!(
            win2.winref.borrow().window.borrow().textdata,
            "Below ground. Look!"
        );
    }

    #[test]
    fn can_read_byte_from_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        win1.winref
            .borrow()
            .window
            .borrow_mut()
            .set_input_buffer("testing");
        let stream1 = glk.window_get_stream(&win1);
        assert_eq!(glk.get_char_stream(stream1), Some(b't'));
    }

    #[test]
    fn can_read_byte_buffer_from_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        win1.winref
            .borrow()
            .window
            .borrow_mut()
            .set_input_buffer("testing");
        let stream1 = glk.window_get_stream(&win1);
        assert_eq!(
            glk.get_buffer_stream(stream1, None),
            "testing".chars().map(|c| c as u8).collect::<Vec<_>>()
        );
    }

    #[test]
    fn can_read_a_line_of_bytes_from_a_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        win1.winref
            .borrow()
            .window
            .borrow_mut()
            .set_input_buffer("testing line 1\ntesting line 2\ntesting line 3\n");
        let stream1 = glk.window_get_stream(&win1);

        assert_eq!(
            glk.get_line_stream(stream1, None),
            "testing line 1"
                .chars()
                .map(|c| c as u8)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            glk.get_line_stream(stream1, None),
            "testing line 2"
                .chars()
                .map(|c| c as u8)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn can_read_char_from_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        win1.winref
            .borrow()
            .window
            .borrow_mut()
            .set_input_buffer("testing");
        let stream1 = glk.window_get_stream(&win1);
        assert_eq!(glk.get_char_stream_uni(stream1), Some('t'));
    }

    #[test]
    fn can_read_char_buffer_from_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        win1.winref
            .borrow()
            .window
            .borrow_mut()
            .set_input_buffer("testing");
        let stream1 = glk.window_get_stream(&win1);
        assert_eq!(
            glk.get_buffer_stream_uni(stream1, None),
            "testing".chars().collect::<Vec<_>>()
        );
    }

    #[test]
    fn can_read_a_line_of_chars_from_a_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        win1.winref
            .borrow()
            .window
            .borrow_mut()
            .set_input_buffer("testing line 1\ntesting line 2\ntesting line 3\n");
        let stream1 = glk.window_get_stream(&win1);

        assert_eq!(
            glk.get_line_stream_uni(stream1, None),
            "testing line 1".chars().collect::<Vec<_>>()
        );
        assert_eq!(
            glk.get_line_stream_uni(stream1, None),
            "testing line 2".chars().collect::<Vec<_>>()
        );
    }

    #[test]
    fn can_count_chars_in_output() {
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
        let stream2 = glk.window_get_stream(&win2);

        glk.put_char_stream(stream2, b'0');
        let stream_results = glk.window_close(&win2);
        assert_eq!(stream_results.read_count, 0);
        assert_eq!(stream_results.write_count, 1);
    }
}
