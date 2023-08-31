mod glk_stream;
mod glk_win;

use std::path::Path;

use unicode_normalization::UnicodeNormalization;

use crate::file_stream::{FileRefManager, GlkFileRef};
use crate::gestalt::OutputType;
use crate::keycode::Keycode;
use crate::stream::{GlkStreamID, StreamManager};
use crate::windows::{GlkWindow, WindowManager};
use crate::GlkRock;
use crate::{gestalt::*, GlkFileUsage};

/// The GLK object. TODO: Insert basic usage here
/// This is the API for GLK interpreted as a Rust API.
#[derive(Default)]
pub struct Glk<T: GlkWindow + Default + 'static> {
    win_mgr: WindowManager<T>,
    stream_mgr: StreamManager,
    fileref_mgr: FileRefManager,
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
     * Glk Section 2.6 - Unicode String Normalization
     */

    /// Convert a string to Normalization Form D
    pub fn buffer_canon_decompose_uni(s: &str) -> String {
        s.nfd().collect::<String>()
    }

    /// Convert a string to Normalization Form C
    pub fn buffer_canon_normalize_uni(s: &str) -> String {
        s.nfc().collect::<String>()
    }

    /*
     * Glk Section 6.1 - The Types of File References
     */

    /// Creates a reference to a temporary file. It is always a new file (one which does not yet
    /// exist). The file (once created) will be somewhere out of the player's way
    pub fn fileref_create_temp(
        &mut self,
        usage: GlkFileUsage,
        rock: GlkRock,
    ) -> Option<GlkFileRef> {
        self.fileref_mgr.create_temp_file(usage, rock)
    }

    /// creates a reference to a file with a specific name. The file will be in a fixed location
    /// relevant to your program, and visible to the player
    pub fn fileref_create_by_name<P: AsRef<Path>>(
        &mut self,
        usage: GlkFileUsage,
        name: P,
        rock: GlkRock,
    ) -> Option<GlkFileRef> {
        self.fileref_mgr
            .create_named_file(usage, name.as_ref().to_path_buf(), rock)
    }

    /*
     * Glk Section 6.2 - Other File Reference Functions
     */
    /// Deletes the file referred to by the file reference. It does not destroy the fileref itself.
    pub fn fileref_delete_file(&mut self, filerefid: GlkFileRef) {
        self.fileref_mgr.delete_file_by_id(filerefid);
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
    use crate::{
        windows::{testwin::GlkTestWindow, GlkWindowType},
        GlkFileMode,
    };

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
    fn at_startup_there_is_no_default_stream() {
        let glk = Glk::<GlkTestWindow>::new();
        assert!(glk.stream_get_current().is_none());
    }

    #[test]
    fn can_write_to_a_window_echo_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();

        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        assert!(glk.window_get_echo_stream(win).is_none());

        let win_stream = glk.window_get_stream(win).unwrap();

        let mem_stream = glk.stream_open_memory(vec![0u8; 20], GlkFileMode::Write, 74);

        glk.window_set_echo_stream(win, Some(mem_stream));
        let echo_stream = glk.window_get_echo_stream(win);
        assert_eq!(Some(mem_stream), echo_stream);

        glk.put_buffer_stream(
            win_stream,
            "hello, world!"
                .chars()
                .map(|ch| ch as u8)
                .collect::<Vec<_>>()
                .as_slice(),
        );

        // this should detach the echo stream from the window automatically
        let close = glk.stream_close(mem_stream);
        assert!(close.is_some());
        if let Some((result, Some(bytes))) = close {
            assert_eq!(result.read_count, 0);
            assert_eq!(result.write_count, 13);
            assert_eq!(
                bytes[0..13],
                "hello, world!"
                    .chars()
                    .map(|ch| ch as u8)
                    .collect::<Vec<_>>()
            );
        } else {
            panic!("stream_close() did not return valid results");
        }

        let echo_stream = glk.window_get_echo_stream(win);
        assert!(echo_stream.is_none());
    }
}
