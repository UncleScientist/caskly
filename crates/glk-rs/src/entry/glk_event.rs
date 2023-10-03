use crate::{
    events::GlkEvent,
    windows::{GlkWindow, GlkWindowID},
};

use super::Glk;

impl<T: GlkWindow + Default> Glk<T> {
    /*
     * Glk Section 4 - Events
     */
    /// Block until event arrives
    pub fn select(&mut self) -> GlkEvent {
        self.event_mgr.block_until_event()
    }

    /// check to see if events are available, and return one. Otherwise return GlkEvent::None
    pub fn select_poll(&mut self) -> GlkEvent {
        self.event_mgr.pop_event()
    }

    /*
     * Glk Section 4.2 - Line Input Events
     */

    /// Request a line of Latin-1 characters from a given window
    pub fn request_line_event(&mut self, win: GlkWindowID, buf: &[u8], initlen: usize) {
        let winref = self
            .win_mgr
            .get_ref(win)
            .expect("line input event requested from non-existent window");
        self.event_mgr
            .queue_line_input_request(&winref, buf, initlen);
    }

    /// request a line of unicode codepoint from a given window
    pub fn request_line_event_uni(&mut self, win: GlkWindowID, buf: &[u32], initlen: usize) {
        let winref = self
            .win_mgr
            .get_ref(win)
            .expect("line input event requested from non-existent window");
        self.event_mgr
            .queue_line_input_uni_request(&winref, buf, initlen);
    }

    /*
     * Glk Section 4.4 - Timer Events
     */

    /// Request a timer event to be sent at fixed intervals, or 0 to turn off
    pub fn request_timer_events(&mut self, millisecs: u32) {
        self.event_mgr.set_timer(millisecs)
    }
}

#[cfg(test)]
mod test {
    use crate::windows::testwin::GlkTestWindow;

    use super::*;

    #[test]
    fn can_check_for_events() {
        Glk::<GlkTestWindow>::start(|glk| {
            assert_eq!(glk.select_poll(), GlkEvent::None);
        });
    }
}
