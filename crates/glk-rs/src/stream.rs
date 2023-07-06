use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type GlkStreamID = u32;

#[derive(Default, Debug)]
pub struct StreamManager<T: StreamHandler> {
    stream: HashMap<u32, GlkStream<T>>,
    val: GlkStreamID,
}

impl<T: StreamHandler + Default> StreamManager<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn new_stream(&mut self) -> GlkStreamID {
        self.stream.insert(self.val, GlkStream::<T>::default());
        self.val += 1;
        self.val - 1
    }

    pub(crate) fn get(&self, id: GlkStreamID) -> Option<GlkStream<T>> {
        let stream = self.stream.get(&id)?;
        Some(GlkStream {
            output: Rc::clone(&stream.output),
        })
    }
}

#[derive(Debug, Default)]
pub struct GlkStream<T: StreamHandler> {
    output: Rc<RefCell<T>>,
}

impl<T: StreamHandler> GlkStream<T> {
    pub fn put_char(&self, ch: u8) {
        println!("glkstream: put char");
        self.output.borrow_mut().put_char(ch);
    }
}

pub trait StreamHandler {
    fn put_char(&mut self, ch: u8);
    fn put_string(&self, s: &str);
    fn put_buffer(&self, buf: &[u8]);
    fn put_char_uni(&self, ch: char);
    // note: put_string_uni() is not here because put_string() handles it
    fn put_buffer_uni(&self, buf: &[char]);
}
