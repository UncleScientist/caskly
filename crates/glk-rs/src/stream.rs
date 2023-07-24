use std::fmt::Debug;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

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
    pub(crate) fn new_stream(&mut self, stream: Rc<RefCell<dyn StreamHandler>>) -> GlkStreamID {
        self.stream.insert(self.val, GlkStream::new(&stream));
        self.val += 1;
        self.val - 1
    }

    pub(crate) fn get(&self, id: GlkStreamID) -> Option<GlkStream> {
        let stream = self.stream.get(&id)?;
        Some(GlkStream {
            sh: Rc::clone(&stream.sh),
        })
    }

    pub(crate) fn close(&mut self, id: GlkStreamID) -> Option<GlkStreamResult> {
        let stream = self.stream.remove(&id)?;
        Some(stream.get_results())
    }
}

#[derive(Debug)]
pub struct GlkStream {
    sh: Rc<RefCell<dyn StreamHandler>>,
}

impl GlkStream {
    pub(crate) fn new(stream: &Rc<RefCell<dyn StreamHandler>>) -> Self {
        Self {
            sh: Rc::clone(stream),
        }
    }

    pub fn put_char(&self, ch: u8) {
        self.sh.borrow_mut().put_char(ch);
        self.sh.borrow_mut().increment_output_count(1);
    }

    pub fn put_string(&self, s: &str) {
        self.sh.borrow_mut().put_string(s);
        self.sh.borrow_mut().increment_output_count(s.len());
    }

    pub fn put_buffer(&self, buf: &[u8]) {
        self.sh.borrow_mut().put_buffer(buf);
        self.sh.borrow_mut().increment_output_count(buf.len());
    }

    pub fn put_char_uni(&self, ch: char) {
        self.sh.borrow_mut().put_char_uni(ch);
        self.sh.borrow_mut().increment_output_count(4);
    }

    pub fn put_buffer_uni(&self, buf: &[char]) {
        self.sh.borrow_mut().put_buffer_uni(buf);
        self.sh.borrow_mut().increment_output_count(4 * buf.len());
    }

    pub fn get_char(&self) -> Option<u8> {
        let ch = self.sh.borrow().get_char();
        if ch.is_some() {
            self.sh.borrow_mut().increment_input_count(1);
        }
        ch
    }

    pub fn get_buffer(&self, maxlen: Option<usize>) -> Vec<u8> {
        let result = self.sh.borrow().get_buffer(maxlen);
        self.sh.borrow_mut().increment_input_count(result.len());
        result
    }

    pub fn get_line(&self, maxlen: Option<usize>) -> Vec<u8> {
        let result = self.sh.borrow().get_line(maxlen);
        self.sh.borrow_mut().increment_input_count(result.len());
        result
    }

    pub fn get_char_uni(&self) -> Option<char> {
        let ch = self.sh.borrow().get_char_uni();
        if ch.is_some() {
            self.sh.borrow_mut().increment_input_count(4);
        }
        ch
    }

    pub fn get_buffer_uni(&self, maxlen: Option<usize>) -> Vec<char> {
        let result = self.sh.borrow().get_buffer_uni(maxlen);
        self.sh.borrow_mut().increment_input_count(result.len() * 4);
        result
    }

    pub fn get_line_uni(&self, maxlen: Option<usize>) -> Vec<char> {
        let result = self.sh.borrow().get_line_uni(maxlen);
        self.sh.borrow_mut().increment_input_count(result.len() * 4);
        result
    }

    pub fn is_window_stream(&self) -> bool {
        self.sh.borrow().is_window_stream()
    }

    pub fn is_memory_stream(&self) -> bool {
        self.sh.borrow().is_memory_stream()
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.sh.borrow().get_data()
    }

    pub fn get_results(&self) -> GlkStreamResult {
        self.sh.borrow().get_results()
    }
}

pub trait StreamHandler: Debug {
    fn put_char(&mut self, ch: u8);
    fn put_string(&mut self, s: &str);
    fn put_buffer(&mut self, buf: &[u8]);
    fn put_char_uni(&mut self, ch: char);
    // note: put_string_uni() is not here because put_string() handles it
    fn put_buffer_uni(&mut self, buf: &[char]);

    fn get_char(&self) -> Option<u8>;
    fn get_buffer(&self, maxlen: Option<usize>) -> Vec<u8>;
    fn get_line(&self, maxlen: Option<usize>) -> Vec<u8>;
    fn get_char_uni(&self) -> Option<char>;
    fn get_buffer_uni(&self, maxlen: Option<usize>) -> Vec<char>;
    fn get_line_uni(&self, maxlen: Option<usize>) -> Vec<char>;

    fn get_data(&self) -> Vec<u8>;

    fn is_window_stream(&self) -> bool;
    fn is_memory_stream(&self) -> bool;
    fn increment_output_count(&mut self, bytes: usize);
    fn increment_input_count(&mut self, bytes: usize);
    fn get_results(&self) -> GlkStreamResult;
}
