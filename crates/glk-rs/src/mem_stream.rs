use std::cell::RefCell;

use crate::stream::{GlkStreamResult, StreamHandler};

#[derive(Debug, Default)]
pub(crate) struct MemStream {
    buf: Vec<u8>,
    cursor: RefCell<usize>,
    result: GlkStreamResult,
}

impl MemStream {
    pub(crate) fn new(buf: Vec<u8>) -> Self {
        Self {
            buf: Vec::from(buf),
            ..Self::default()
        }
    }
}

impl StreamHandler for MemStream {
    fn put_char(&mut self, ch: u8) {
        if *self.cursor.borrow() < self.buf.len() {
            self.buf[*self.cursor.borrow()] = ch;
            *self.cursor.borrow_mut() += 1;
        }
    }

    fn put_char_uni(&mut self, ch: char) {
        let chu32 = ch as u32;
        self.put_char((chu32 >> 24) as u8);
        self.put_char(((chu32 >> 16) & 0xff) as u8);
        self.put_char(((chu32 >> 8) & 0xff) as u8);
        self.put_char((chu32 & 0xff) as u8);
    }

    fn put_string(&mut self, s: &str) {
        for ch in s.chars() {
            self.put_char_uni(ch);
        }
    }

    fn put_buffer(&mut self, buf: &[u8]) {
        for byte in buf {
            self.put_char(*byte);
        }
    }

    fn put_buffer_uni(&mut self, buf: &[char]) {
        for ch in buf {
            self.put_char_uni(*ch);
        }
    }

    fn get_char(&self) -> Option<u8> {
        if *self.cursor.borrow() < self.buf.len() {
            *self.cursor.borrow_mut() += 1;
            Some(self.buf[*self.cursor.borrow() - 1])
        } else {
            None
        }
    }

    fn get_buffer(&self, _maxlen: Option<usize>) -> Vec<u8> {
        Vec::new()
    }

    fn get_line(&self, _maxlen: Option<usize>) -> Vec<u8> {
        Vec::new()
    }

    fn get_char_uni(&self) -> Option<char> {
        None
    }

    fn get_buffer_uni(&self, _maxlen: Option<usize>) -> Vec<char> {
        Vec::new()
    }

    fn get_line_uni(&self, _maxlen: Option<usize>) -> Vec<char> {
        Vec::new()
    }

    fn get_data(&self) -> Vec<u8> {
        self.buf.clone()
    }

    fn is_window_stream(&self) -> bool {
        false
    }

    fn is_memory_stream(&self) -> bool {
        true
    }

    fn increment_output_count(&mut self, count: usize) {
        self.result.write_count += count as u32;
    }

    fn increment_input_count(&mut self, count: usize) {
        self.result.read_count += count as u32;
    }

    fn get_results(&self) -> GlkStreamResult {
        self.result.clone()
    }
}
