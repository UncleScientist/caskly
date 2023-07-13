use std::fmt::Debug;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type GlkStreamID = u32;

#[derive(Default, Debug)]
pub struct StreamManager {
    stream: HashMap<u32, GlkStream>,
    val: GlkStreamID,
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
    }

    pub fn put_string(&self, s: &str) {
        self.sh.borrow_mut().put_string(s);
    }

    pub fn put_buffer(&self, buf: &[u8]) {
        self.sh.borrow_mut().put_buffer(buf);
    }

    pub fn put_char_uni(&self, ch: char) {
        self.sh.borrow_mut().put_char_uni(ch);
    }

    pub fn put_buffer_uni(&self, buf: &[char]) {
        self.sh.borrow_mut().put_buffer_uni(buf);
    }

    pub fn get_char(&self) -> Option<u8> {
        self.sh.borrow().get_char()
    }

    pub fn get_buffer(&self, maxlen: Option<usize>) -> Vec<u8> {
        self.sh.borrow().get_buffer(maxlen)
    }

    pub fn get_line(&self, maxlen: Option<usize>) -> Vec<u8> {
        self.sh.borrow().get_line(maxlen)
    }

    pub fn get_char_uni(&self) -> Option<char> {
        self.sh.borrow().get_char_uni()
    }

    pub fn get_buffer_uni(&self, maxlen: Option<usize>) -> Vec<char> {
        self.sh.borrow().get_buffer_uni(maxlen)
    }

    pub fn get_line_uni(&self, maxlen: Option<usize>) -> Vec<char> {
        self.sh.borrow().get_line_uni(maxlen)
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
}
