use std::cell::RefCell;

use crate::{
    stream::{GlkStreamHandler, GlkStreamID},
    GlkSeekMode,
};

#[derive(Debug, Default)]
pub(crate) struct MemStream {
    buf: Vec<u8>,
    cursor: RefCell<usize>,
}

impl MemStream {
    pub(crate) fn new(buf: Vec<u8>) -> Self {
        Self {
            buf,
            ..Self::default()
        }
    }

    fn get_bytes(&mut self, maxlen: Option<usize>, end_char: Option<u8>) -> Vec<u8> {
        let remaining_bytes = self.buf.len() - *self.cursor.borrow();
        let count = if let Some(max) = maxlen {
            max.min(remaining_bytes)
        } else {
            remaining_bytes
        };

        let mut result = Vec::new();
        for _ in 0..count {
            if let Some(ch) = self.get_char() {
                if Some(ch) == end_char {
                    break;
                }
                result.push(ch);
            }
        }

        result
    }

    fn get_uni(&mut self, maxlen: Option<usize>, end_char: Option<char>) -> String {
        let remaining_bytes = self.buf.len() - *self.cursor.borrow();
        let count = if let Some(max) = maxlen {
            max.min(remaining_bytes / 4)
        } else {
            remaining_bytes / 4
        };

        let mut result = String::new();
        for _ in 0..count {
            if let Some(ch) = self.get_char_uni() {
                if Some(ch) == end_char {
                    break;
                }
                result.push(ch);
            }
        }

        result
    }
}

impl GlkStreamHandler for MemStream {
    fn get_echo_stream(&self) -> Option<GlkStreamID> {
        None
    }

    fn close(&mut self) {}

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

    fn get_char(&mut self) -> Option<u8> {
        if *self.cursor.borrow() < self.buf.len() {
            *self.cursor.borrow_mut() += 1;
            Some(self.buf[*self.cursor.borrow() - 1])
        } else {
            None
        }
    }

    fn get_buffer(&mut self, maxlen: Option<usize>) -> Vec<u8> {
        self.get_bytes(maxlen, None)
    }

    fn get_line(&mut self, maxlen: Option<usize>) -> Vec<u8> {
        self.get_bytes(maxlen, Some(b'\n'))
    }

    fn get_char_uni(&mut self) -> Option<char> {
        let mut result = 0u32;
        for _ in 0..4 {
            result = (result << 8) | (self.get_char()? as u32);
        }

        char::from_u32(result)
    }

    fn get_buffer_uni(&mut self, maxlen: Option<usize>) -> String {
        self.get_uni(maxlen, None)
    }

    fn get_line_uni(&mut self, maxlen: Option<usize>) -> String {
        self.get_uni(maxlen, Some('\n'))
    }

    fn get_position(&self) -> u32 {
        *self.cursor.borrow() as u32
    }

    fn set_position(&mut self, pos: i32, seekmode: crate::GlkSeekMode) -> Option<()> {
        let new_cursor = match seekmode {
            GlkSeekMode::Start => pos,
            GlkSeekMode::Current => *self.cursor.borrow() as i32 + pos,
            GlkSeekMode::End => self.buf.len() as i32 + pos,
        };

        if new_cursor < 0 || new_cursor > self.buf.len() as i32 {
            None
        } else {
            *self.cursor.borrow_mut() = new_cursor as usize;
            Some(())
        }
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
}
