use std::{
    sync::mpsc::Sender,
    thread,
    time::{Duration, Instant},
};

use rglk::prelude::*;

fn main() {
    Glk::<SimpleWindow>::start(|glk| {
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        println!("created window {win:?}");

        let winstream = glk.window_get_stream(win).unwrap();
        glk.put_string_stream(winstream, "hello, world!\n");
        let results = glk.window_close(win).unwrap();

        println!(
            "read = {}, wrote = {}",
            results.read_count, results.write_count
        );

        glk.request_timer_events(250);
        thread::sleep(Duration::from_secs(1));
        assert_eq!(glk.select_poll(), GlkEvent::Timer);
        glk.request_timer_events(0);

        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        println!("created window {win:?}");

        println!("enter a line of text");
        let buf = [0u32; 80];
        glk.request_line_event_uni(win, &buf, 0);
        match glk.select() {
            GlkEvent::LineInput { win, buf } => {
                println!("window {win} sent line input event: {buf:?}")
            }
            x => panic!("got {x:?} instead of a line input"),
        }

        assert_eq!(glk.select_poll(), GlkEvent::None);

        glk.request_timer_events(1000);
        thread::sleep(Duration::from_millis(1500));
        glk.request_timer_events(0);
        assert_eq!(glk.select_poll(), GlkEvent::None);

        glk.request_timer_events(1000);
        for _ in 0..3 {
            let event = glk.select();
            println!("{:?} {:?}", Instant::now(), event);
        }

        glk.request_timer_events(100_000);
        thread::sleep(Duration::from_secs(3));
        assert_eq!(glk.select_poll(), GlkEvent::None);

        glk.request_timer_events(1000);
        let event = glk.select();
        println!("{:?} select returned: {:?}", Instant::now(), event);

        println!("{:?} delaying...", Instant::now());
        thread::sleep(Duration::from_secs(5));
        println!("{:?} delay finished", Instant::now());

        for _ in 0..5 {
            let event = glk.select();
            println!("{:?} {:?}", Instant::now(), event);
        }

        println!("Done");
    });

    // main event loop
}

#[derive(Debug, Default)]
struct SimpleWindow {
    winid: GlkWindowID,
}

impl GlkWindow for SimpleWindow {
    fn init(&mut self, winid: GlkWindowID) {
        self.winid = winid;
        println!("init window {winid}");
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

    fn get_line(&mut self, event: LineInput, _initlen: usize, tx: Sender<GlkEvent>) {
        let win = self.winid;
        println!("get line from {win}");
        let _ = thread::spawn(move || {
            let is_latin1 = match event {
                LineInput::Latin1(val) => {
                    println!(
                        "{}",
                        val.iter().map(|byte| *byte as char).collect::<String>()
                    );
                    true
                }
                LineInput::Unicode(val) => {
                    println!(
                        "{}",
                        val.iter()
                            .map(|long| char::from_u32(*long).unwrap())
                            .collect::<String>()
                    );
                    false
                }
            };
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line); // <- convert to actual readline
            println!(">>> read '{line}' <<<");
            let _ = tx.send(if is_latin1 {
                GlkEvent::LineInput {
                    win,
                    buf: LineInput::Latin1(line.into()),
                }
            } else {
                GlkEvent::LineInput {
                    win,
                    buf: LineInput::Unicode(line.chars().map(|ch| ch as u32).collect::<Vec<_>>()),
                }
            });
        });
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
