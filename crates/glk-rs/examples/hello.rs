use std::{
    thread,
    time::{Duration, Instant},
};

use rglk::prelude::*;

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

    fn write_char(&mut self, ch: u8) -> usize {
        print!("{ch}");
        1
    }

    fn write_string(&mut self, s: &str) -> usize {
        print!("{s}");
        s.len()
    }

    fn write_buffer(&mut self, buf: &[u8]) -> usize {
        buf.iter().map(|byte| self.write_char(*byte)).sum()
    }

    fn write_char_uni(&mut self, ch: char) -> usize {
        print!("{ch}");
        4
    }

    fn write_buffer_uni(&mut self, buf: &[char]) -> usize {
        buf.iter().map(|ch| self.write_char_uni(*ch)).sum()
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

    println!("{:?}", glk.select());

    assert_eq!(glk.select_poll(), GlkEvent::None);
    glk.request_timer_events(1000);
    thread::sleep(Duration::from_millis(1500));
    glk.request_timer_events(0);
    assert_eq!(glk.select_poll(), GlkEvent::Timer);
    assert_eq!(glk.select_poll(), GlkEvent::None);

    glk.request_timer_events(1000);
    for _ in 0..3 {
        println!("{:?} {:?}", Instant::now(), glk.select());
    }

    glk.request_timer_events(100_000);
    thread::sleep(Duration::from_secs(3));
    glk.request_timer_events(1000);
    println!("{:?} select returned: {:?}", Instant::now(), glk.select());

    println!("{:?} delaying...", Instant::now());
    thread::sleep(Duration::from_secs(5));
    println!("{:?} delay finished", Instant::now());

    for _ in 0..5 {
        println!("{:?} {:?}", Instant::now(), glk.select());
    }

    println!("Done");
}
