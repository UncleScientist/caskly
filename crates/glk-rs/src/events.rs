use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    time::{Duration, Instant},
};

use crate::{
    keycode::Keycode,
    windows::{GlkWindow, GlkWindowID, WindowRef},
};

/// A line input event - either Latin-1 characters, or Unicode codepoints
#[derive(PartialEq, Debug)]
pub enum LineInput {
    /// Latin-1
    Latin1(Vec<u8>),
    /// Unicode
    Unicode(Vec<u32>),
}

/// Events
#[derive(PartialEq, Debug)]
pub enum GlkEvent {
    /// Used by glk_select_poll() to indicate that no events are pending
    None,

    /// A timer event
    Timer,

    /// Character input from a window
    CharInput {
        /// The window generating the key event
        win: GlkWindowID,
        /// the key that was pressed
        key: Keycode,
    },

    /// Line input from a window
    LineInput {
        /// The window generating the line event
        win: GlkWindowID,

        /// The line that was read
        buf: LineInput,
    },

    /// A mouse event from a text grid or graphics window
    Mouse {
        /// The window generating the mouse event
        win: GlkWindowID,
        /// The x-coordinate of the event
        x: u32,
        /// The y-coordinate of the event
        y: u32,
    },

    /// A window (and its children) were rearranged
    Arrange {
        /// The window requesting rearrangement
        win: GlkWindowID,
    },

    /// A window (and its children) need to be redrawn
    Redraw {
        /// The window that needs to be redrawn
        win: GlkWindowID,
    },

    /// A hyperlink was clicked/selected
    Hyperlink {
        /// The window generating the hyperlink event
        win: GlkWindowID,
        /// The user's linkval for the selected link
        linkval: u32,
    },

    /// A sound finished playing
    SoundNotify {
        /// the Sound Resource (from the blorb file) that finished playing
        resource_id: u32,
        /// The user's notification value
        notify: u32,
    },

    /// A volume change was completed
    VolumeNotify {
        /// The user's notification value
        notify: u32,
    },
}

pub(crate) struct EventManager {
    pending: VecDeque<GlkEvent>,
    last_timer_event: Instant,
    timer_interval: Duration,
    tx: Sender<GlkEvent>,
    rx: Receiver<GlkEvent>,
}

impl Default for EventManager {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            pending: VecDeque::new(),
            last_timer_event: Instant::now(),
            timer_interval: Duration::from_millis(0),
            tx,
            rx,
        }
    }
}

impl EventManager {
    fn fill_event_queue(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            self.pending.push_back(event);
        }
    }

    fn time_left(&self) -> Duration {
        let now = Instant::now();
        (self.last_timer_event + self.timer_interval) - now
    }

    // This will check for an event and return it. If no events are available,
    // then return GlkEvent::None
    pub(crate) fn pop_event(&mut self) -> GlkEvent {
        self.fill_event_queue();

        if let Some(event) = self.pending.pop_front() {
            return event;
        }

        if !self.timer_interval.is_zero() && self.time_left().is_zero() {
            self.last_timer_event = Instant::now();
            return GlkEvent::Timer;
        }

        GlkEvent::None
    }

    // This will block until an event is available, and then return it. Should never
    // return GlkEvent::None
    pub(crate) fn block_until_event(&mut self) -> GlkEvent {
        self.fill_event_queue();

        let event = self.pop_event();
        if event != GlkEvent::None {
            return event;
        }

        if !self.timer_interval.is_zero() {
            let timeout = self.time_left();
            match self.rx.recv_timeout(timeout) {
                Ok(event) => event,
                Err(RecvTimeoutError::Timeout) => {
                    self.last_timer_event = Instant::now();
                    GlkEvent::Timer
                }
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("library bug: tx disconnected");
                }
            }
        } else {
            self.rx
                .recv()
                .expect("library bug: should have gotten an event")
        }
    }

    pub(crate) fn set_timer(&mut self, ms: u32) {
        self.timer_interval = Duration::from_millis(ms as u64);
    }

    pub(crate) fn queue_line_input_request<T: GlkWindow + Default>(
        &mut self,
        winref: &WindowRef<T>,
        buf: &[u8],
        initlen: usize,
    ) {
        let input = LineInput::Latin1(Vec::from(buf));
        winref.get_line(input, initlen, self.tx.clone());
    }

    pub(crate) fn queue_line_input_uni_request<T: GlkWindow + Default>(
        &mut self,
        winref: &WindowRef<T>,
        buf: &[u32],
        initlen: usize,
    ) {
        let input = LineInput::Unicode(Vec::from(buf));
        winref.get_line(input, initlen, self.tx.clone());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_create_an_event() {
        let foo = GlkEvent::None;
        assert_eq!(foo, GlkEvent::None);
    }
}
