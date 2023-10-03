mod util;
use util::win::SimpleWindow;

use rglk::prelude::*;

fn main() {
    Glk::<SimpleWindow>::start(|glk| {
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();

        let wsm = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(40),
            border: false,
        };

        let splitter = glk
            .window_open(Some(win), GlkWindowType::TextBuffer, Some(wsm), 123)
            .expect("unable to generate split window");

        let win_stream = glk.window_get_stream(win).unwrap();
        let split_stream = glk.window_get_stream(splitter).unwrap();

        glk.put_string_stream(win_stream, "This goes to the first window");
        glk.put_string_stream(split_stream, "This goes to the second window");

        glk.window_close(win);
    });
}
