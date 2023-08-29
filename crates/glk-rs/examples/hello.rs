use rglk::stream::GlkStreamHandler;
use rglk::windows::{GlkWindow, GlkWindowSize, GlkWindowType};
use rglk::Glk;

#[derive(Debug, Default)]
struct SimpleWindow;

impl GlkWindow for SimpleWindow {
    fn get_size(&self) -> GlkWindowSize {
        todo!()
    }

    fn move_cursor(&mut self, _x: u32, _y: u32) {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }
}

impl GlkStreamHandler for SimpleWindow {
    fn put_char(&mut self, _ch: u8) {
        todo!()
    }

    fn put_string(&mut self, s: &str) {
        print!("{s}");
    }

    fn put_buffer(&mut self, _buf: &[u8]) {
        todo!()
    }

    fn put_char_uni(&mut self, _ch: char) {
        todo!()
    }

    fn put_buffer_uni(&mut self, _buf: &[char]) {
        todo!()
    }

    fn get_char(&mut self) -> Option<u8> {
        todo!()
    }

    fn get_buffer(&mut self, _maxlen: Option<usize>) -> Vec<u8> {
        todo!()
    }

    fn get_line(&mut self, _maxlen: Option<usize>) -> Vec<u8> {
        todo!()
    }

    fn get_char_uni(&mut self) -> Option<char> {
        todo!()
    }

    fn get_buffer_uni(&mut self, _maxlen: Option<usize>) -> String {
        todo!()
    }

    fn get_line_uni(&mut self, _maxlen: Option<usize>) -> String {
        todo!()
    }

    fn get_position(&self) -> u32 {
        todo!()
    }

    fn set_position(&mut self, _pos: i32, _seekmode: rglk::GlkSeekMode) -> Option<()> {
        todo!()
    }

    fn get_data(&self) -> Vec<u8> {
        todo!()
    }

    fn close(&mut self) {
        // nop
    }

    fn is_window_stream(&self) -> bool {
        todo!()
    }

    fn is_memory_stream(&self) -> bool {
        todo!()
    }
}

fn main() {
    let mut glk = Glk::<SimpleWindow>::new();

    let win = glk
        .window_open(None, GlkWindowType::TextBuffer, None, 73)
        .unwrap();

    let winstream = glk.window_get_stream(win).unwrap();
    glk.put_string_stream(winstream, "hello, world!\n");
    let results = glk.window_close(win).unwrap();

    println!(
        "read = {}, wrote = {}",
        results.read_count, results.write_count
    );
}
