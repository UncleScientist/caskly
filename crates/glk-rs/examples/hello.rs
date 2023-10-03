mod util;
use util::win::SimpleWindow;

use std::{
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
        glk.put_char_stream(winstream, b'%');
        glk.put_string_stream(winstream, "\n");
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
