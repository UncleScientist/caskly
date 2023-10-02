mod glk_clock;
mod glk_event;
mod glk_stream;
mod glk_win;

use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use unicode_normalization::UnicodeNormalization;

use crate::events::{EventManager, GlkEvent};
use crate::file_stream::{FileRefManager, GlkFileRef};
use crate::gestalt::OutputType;
use crate::keycode::Keycode;
use crate::prelude::GlkRock;
use crate::stream::{GlkStreamID, StreamManager};
use crate::windows::{GlkWindow, GlkWindowID, WindowManager};
use crate::{gestalt::*, GlkFileUsage};

/// A request from the glk library to the window code for something to happen
pub enum GlkMessage {
    /// write a string to a stdio stream or window
    Write {
        /// winid: the window to write to
        winid: GlkWindowID,
        /// message: the message to write to the window
        message: String,
    },
}

/// The result of a request from glk
#[derive(Debug)]
pub enum GlkResult {
    /// the request worked, here's the answer
    Success(GlkEvent),

    /// how many characters were written to output
    Result(usize),
}

/// The GLK object. TODO: Insert basic usage here
/// This is the API for GLK interpreted as a Rust API.
///
/// Messages that the window subsystem needs to handle
/// - gestalt::LineInput(ch)
/// - gestalt::CharInput(ch)
/// - gestalt::LineInputEcho
/// - gestalt::LineTerminators
/// - gestalt::LineTerminatorKey(ch)
/// - gestalt::MouseInput
/// - gestalt::Graphics
/// - gestalt::DrawImage
/// - gestalt::GraphicsTransparency
/// - gestalt::GraphicsCharInput
/// - gestalt::Hyperlinks
/// - gestalt::HyperlinkInput
/// - glk_window_open(parent, method, rock)
/// - glk_window_close(window_id) -> StreamResult
/// - glk_window_get_size(window_id)
/// - glk_window_get/set_arrangement(window_id[, window_info]) -> WindowInfo
/// - glk_window_clear(window_id)
/// - glk_request_char_event(window_id)     -- & char_event_uni()?
/// - glk_cancel_char_event(window_id)
/// - glk_request_line_event(window_id)     -- & line_event_uni()?
/// - glk_cancel_line_event(window_id)
/// - glk_set_echo_line_event(window_id, bool)
/// - glk_set_terminators_line_event(window_id, Vec<keycode>)
/// - glk_request/cancel_mouse_event(window_id)
/// - glk_put_char(window_id, ch)
/// - glk_put_string(window_id, string)     -- & put_buffer()?
/// - glk_set_style(window_id, style)
/// - glk_stylehint_set(window_id, style, hint, val)
/// - glk_stylehint_clear(window_id, style, hint)
/// - glk_style_distinguish(window_id, style1, style2)
/// - glk_style_measure(window_id, style, hint) -> MeasurementResult
/// - glk_image_draw(window_id, image, pos)     -- [pos: x/y coords, or ImageAlign value]
/// - glk_image_draw_scaled(window_id, image, pos, scale)
/// - glk_window_set_background_color(window_id, color)
/// - glk_window_fill_rect(window_id, color, rect)
/// - glk_window_erase_rect(window_id, rect)
/// - glk_window_flow_break(window_id)
/// - glk_set_hyperlink(window_id, linkval)
/// - glk_request_hyperlink_event(window_id)
/// - glk_cancel_hyperlink_event(window_id)

#[derive(Default)]
pub struct Glk<T: GlkWindow + Default + 'static> {
    win_mgr: WindowManager<T>,
    event_mgr: EventManager,
    stream_mgr: StreamManager,
    fileref_mgr: FileRefManager,
    default_stream: Option<GlkStreamID>,
    command: Option<Sender<GlkMessage>>,
    response: Option<Receiver<GlkResult>>,
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

impl<T: GlkWindow + Default + 'static> Glk<T> {
    /// Create a new glk interface
    pub fn new(command: Sender<GlkMessage>, response: Receiver<GlkResult>) -> Self {
        Self {
            command: Some(command),
            response: Some(response),
            ..Self::default()
        }
    }

    /// start up a glk-based i/o subsystem
    pub fn start<F: FnOnce(&mut Glk<T>) + Send + 'static>(func: F) {
        let (command, request) = mpsc::channel(); // glk:command.send(), win:request.recv()
        let (result, response) = mpsc::channel(); // glk:response.recv(), win:result.send()

        let joiner = thread::spawn(move || {
            let mut glk = Glk::<T>::new(command, response);
            func(&mut glk);
        });

        let mut window_system = T::new(request, result);
        window_system.run();

        let _ = joiner.join();
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
    use crate::windows::testwin::GlkTestWindow;

    #[test]
    fn can_get_glk_version() {
        /*
        let glk = Glk::<GlkTestWindow>::new();
        assert_eq!(
            GestaltResult::Version(0x00000705),
            glk.gestalt(Gestalt::Version)
        );
        */
        Glk::<GlkTestWindow>::start(|glk| {
            assert_eq!(
                GestaltResult::Version(0x00000705),
                glk.gestalt(Gestalt::Version)
            )
        });
    }
    /*

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
    */
}
