use std::fmt::Debug;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{GlkFileMode, GlkRock, GlkSeekMode};

/// An opaque stream ID
pub type GlkStreamID = u32;

#[derive(Default)]
pub(crate) struct StreamManager {
    stream: HashMap<GlkStreamID, GlkStream>,
    val: GlkStreamID,
}

/// The stats from the stream that is being closed
#[derive(Debug, Default, Clone)]
pub struct GlkStreamResult {
    /// number of characters that were read from this stream
    pub read_count: u32,
    /// number of characters that were written to this stream
    pub write_count: u32,
}

impl StreamManager {
    pub(crate) fn new_stream(
        &mut self,
        stream: Rc<RefCell<dyn GlkStreamHandler>>,
        mode: GlkFileMode,
    ) -> GlkStreamID {
        self.stream
            .insert(self.val, GlkStream::new(&stream, mode, 0));
        self.val += 1;
        self.val - 1
    }

    pub(crate) fn get(&mut self, id: GlkStreamID) -> Option<&mut GlkStream> {
        self.stream.get_mut(&id)
    }

    pub(crate) fn close(&mut self, id: GlkStreamID) -> Option<GlkStreamResult> {
        let stream = self.stream.remove(&id)?;
        stream.sh.borrow_mut().close();
        Some(stream.get_results())
    }
}

pub(crate) struct GlkStream {
    sh: Rc<RefCell<dyn GlkStreamHandler>>,
    mode: GlkFileMode,
    _rock: GlkRock,
    read_count: usize,
    write_count: usize,
}

impl GlkStream {
    pub(crate) fn new(
        stream: &Rc<RefCell<dyn GlkStreamHandler>>,
        mode: GlkFileMode,
        _rock: GlkRock,
    ) -> Self {
        Self {
            sh: Rc::clone(stream),
            mode,
            _rock,
            read_count: 0,
            write_count: 0,
        }
    }

    fn check_write(&self) -> bool {
        if matches!(
            self.mode,
            GlkFileMode::Write | GlkFileMode::ReadWrite | GlkFileMode::WriteAppend
        ) {
            true
        } else {
            panic!("cannot write to a non-writable stream");
        }
    }

    fn check_read(&self) -> bool {
        if matches!(self.mode, GlkFileMode::Read | GlkFileMode::ReadWrite) {
            true
        } else {
            panic!("cannot read from a non-readable stream");
        }
    }

    pub fn put_char(&mut self, ch: u8) {
        self.check_write();
        self.write_count += self.sh.borrow_mut().put_char(ch);
    }

    pub fn put_string(&mut self, s: &str) {
        self.check_write();
        self.write_count += self.sh.borrow_mut().put_string(s);
    }

    pub fn put_buffer(&mut self, buf: &[u8]) {
        self.check_write();
        self.write_count += self.sh.borrow_mut().put_buffer(buf);
    }

    pub fn put_char_uni(&mut self, ch: char) {
        self.check_write();
        self.write_count += self.sh.borrow_mut().put_char_uni(ch);
    }

    pub fn put_buffer_uni(&mut self, buf: &[char]) {
        self.check_write();
        self.write_count += self.sh.borrow_mut().put_buffer_uni(buf);
    }

    pub fn get_char(&mut self) -> Option<u8> {
        self.check_read();
        let ch = self.sh.borrow_mut().get_char();
        if ch.is_some() {
            self.read_count += 1;
        }
        ch
    }

    pub fn get_buffer(&mut self, maxlen: Option<usize>) -> Vec<u8> {
        self.check_read();
        let result = self.sh.borrow_mut().get_buffer(maxlen);
        self.read_count += result.len();
        result
    }

    pub fn get_line(&mut self, maxlen: Option<usize>) -> Vec<u8> {
        self.check_read();
        let result = self.sh.borrow_mut().get_line(maxlen);
        self.read_count += result.len();
        result
    }

    pub fn get_char_uni(&mut self) -> Option<char> {
        self.check_read();
        let ch = self.sh.borrow_mut().get_char_uni();
        if ch.is_some() {
            self.read_count += 4;
        }
        ch
    }

    pub fn get_buffer_uni(&mut self, maxlen: Option<usize>) -> String {
        self.check_read();
        let result = self.sh.borrow_mut().get_buffer_uni(maxlen);
        self.read_count += result.len() * 4;
        result
    }

    pub fn get_line_uni(&mut self, maxlen: Option<usize>) -> String {
        self.check_read();
        let result = self.sh.borrow_mut().get_line_uni(maxlen);
        self.read_count += result.len() * 4;
        result
    }

    pub fn is_window_stream(&self) -> bool {
        self.sh.borrow().is_window_stream()
    }

    pub fn is_memory_stream(&self) -> bool {
        self.sh.borrow().is_memory_stream()
    }

    pub fn get_position(&self) -> u32 {
        self.sh.borrow().get_position()
    }

    pub fn set_position(&self, pos: i32, mode: GlkSeekMode) -> Option<()> {
        self.sh.borrow_mut().set_position(pos, mode)
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.sh.borrow().get_data()
    }

    pub fn get_results(&self) -> GlkStreamResult {
        GlkStreamResult {
            read_count: self.read_count as u32,
            write_count: self.write_count as u32,
        }
    }

    pub fn get_echo_stream(&self) -> Option<GlkStreamID> {
        self.sh.borrow().get_echo_stream()
    }

    // internal helper functions
    pub(crate) fn char_to_bytestream(ch: char) -> Vec<u8> {
        let mut bytes = [0u8; 4];
        let len = ch.encode_utf8(&mut bytes).len();
        Vec::from_iter(bytes[0..len].iter().copied())
    }
}

pub(crate) trait GlkStreamHandler {
    fn put_char(&mut self, ch: u8) -> usize;
    fn put_string(&mut self, s: &str) -> usize;
    fn put_buffer(&mut self, buf: &[u8]) -> usize;
    fn put_char_uni(&mut self, ch: char) -> usize;
    fn put_buffer_uni(&mut self, buf: &[char]) -> usize;
    // note: put_string_uni() is not here because put_string() handles it

    fn get_char(&mut self) -> Option<u8>;
    fn get_buffer(&mut self, maxlen: Option<usize>) -> Vec<u8>;
    fn get_line(&mut self, maxlen: Option<usize>) -> Vec<u8>;
    fn get_char_uni(&mut self) -> Option<char>;
    fn get_buffer_uni(&mut self, maxlen: Option<usize>) -> String;
    fn get_line_uni(&mut self, maxlen: Option<usize>) -> String;

    fn get_position(&self) -> u32;
    fn set_position(&mut self, pos: i32, seekmode: GlkSeekMode) -> Option<()>;

    fn get_data(&self) -> Vec<u8>;
    fn get_echo_stream(&self) -> Option<GlkStreamID>;

    fn close(&mut self);

    fn is_window_stream(&self) -> bool;
    fn is_memory_stream(&self) -> bool;
}
