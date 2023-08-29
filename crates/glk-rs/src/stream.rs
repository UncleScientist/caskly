use std::fmt::Debug;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{GlkFileMode, GlkRock, GlkSeekMode};

/// An opaque stream ID
pub type GlkStreamID = u32;

#[derive(Default, Debug)]
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

#[derive(Debug)]
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
        self.sh.borrow_mut().put_char(ch);
        self.write_count += 1;
    }

    pub fn put_string(&mut self, s: &str) {
        self.check_write();
        self.sh.borrow_mut().put_string(s);
        self.write_count += s.len();
    }

    pub fn put_buffer(&mut self, buf: &[u8]) {
        self.check_write();
        self.sh.borrow_mut().put_buffer(buf);
        self.write_count += buf.len();
    }

    pub fn put_char_uni(&mut self, ch: char) {
        self.check_write();
        self.sh.borrow_mut().put_char_uni(ch);
        self.write_count += 4;
    }

    pub fn put_buffer_uni(&mut self, buf: &[char]) {
        self.check_write();
        self.sh.borrow_mut().put_buffer_uni(buf);
        self.write_count += 4 * buf.len();
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
}

/// Define this for your window type
pub trait GlkStreamHandler: Debug {
    /// Write a byte to a stream
    fn put_char(&mut self, ch: u8);

    /// Write a unicode string to a stream
    fn put_string(&mut self, s: &str);

    /// write an array of bytes to a stream
    fn put_buffer(&mut self, buf: &[u8]);

    /// write a unicode character to a stream
    fn put_char_uni(&mut self, ch: char);
    // note: put_string_uni() is not here because put_string() handles it

    /// write an array of unicode characters to a stream
    fn put_buffer_uni(&mut self, buf: &[char]);

    /// read a byte from a stream
    fn get_char(&mut self) -> Option<u8>;

    /// read an array of bytes from a stream
    fn get_buffer(&mut self, maxlen: Option<usize>) -> Vec<u8>;

    /// read a line of bytes from a stream (up to a newline character)
    fn get_line(&mut self, maxlen: Option<usize>) -> Vec<u8>;

    /// read a unicode character from a stream
    fn get_char_uni(&mut self) -> Option<char>;

    /// read a unicode string from a stream
    fn get_buffer_uni(&mut self, maxlen: Option<usize>) -> String;

    /// read a unicode string up to a newline from a stream
    fn get_line_uni(&mut self, maxlen: Option<usize>) -> String;

    /// get the read/write position of the underlying file
    fn get_position(&self) -> u32;

    /// set the read/write position for the underlying file
    fn set_position(&mut self, pos: i32, seekmode: GlkSeekMode) -> Option<()>;

    /// for memory streams only: retrieve the data buffer
    fn get_data(&self) -> Vec<u8>;

    /// close/finalize a stream
    fn close(&mut self);

    /// return true for window streams
    fn is_window_stream(&self) -> bool;

    /// return true for memory streams
    fn is_memory_stream(&self) -> bool;
}
