use rglk::prelude::*;

fn main() {
    let glk = Glk::<UnimplementedWindow>::new();
    let now = glk.current_time();
    println!("now = {now:?}");

    let utc = glk.time_to_date_utc(&now);
    println!("utc = {utc:?}");
}

#[derive(Default)]
struct UnimplementedWindow;
impl GlkWindow for UnimplementedWindow {
    fn init(&mut self, _winid: GlkWindowID) {
        todo!()
    }

    fn get_size(&self) -> GlkWindowSize {
        todo!()
    }

    fn move_cursor(&mut self, _x: u32, _y: u32) {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }

    fn get_line(
        &mut self,
        _event: LineInput,
        _initlen: usize,
        _tx: std::sync::mpsc::Sender<GlkEvent>,
    ) {
        todo!()
    }

    fn write_char(&mut self, _ch: u8) -> usize {
        todo!()
    }

    fn write_string(&mut self, _s: &str) -> usize {
        todo!()
    }

    fn write_buffer(&mut self, _buf: &[u8]) -> usize {
        todo!()
    }

    fn write_char_uni(&mut self, _ch: char) -> usize {
        todo!()
    }

    fn write_buffer_uni(&mut self, _buf: &[char]) -> usize {
        todo!()
    }
}
