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
            output: Rc::clone(&stream.output),
        })
    }
}

#[derive(Debug)]
pub struct GlkStream {
    output: Rc<RefCell<dyn StreamHandler>>,
}

impl GlkStream {
    pub(crate) fn new(stream: &Rc<RefCell<dyn StreamHandler>>) -> Self {
        Self {
            output: Rc::clone(stream),
        }
    }

    pub fn put_char(&self, ch: u8) {
        println!("glkstream: put char");
        self.output.borrow_mut().put_char(ch);
    }
}

pub trait StreamHandler: Debug {
    fn put_char(&mut self, ch: u8);
    fn put_string(&self, s: &str);
    fn put_buffer(&self, buf: &[u8]);
    fn put_char_uni(&self, ch: char);
    // note: put_string_uni() is not here because put_string() handles it
    fn put_buffer_uni(&self, buf: &[char]);
}
