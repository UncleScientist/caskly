use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    time::{Duration, Instant},
};

use crate::{keycode::Keycode, windows::GlkWindowID};

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
        buf: Vec<u32>,
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
    _tx: Sender<GlkEvent>,
    rx: Receiver<GlkEvent>,
}

impl Default for EventManager {
    fn default() -> Self {
        let (_tx, rx) = mpsc::channel();
        Self {
            pending: VecDeque::new(),
            last_timer_event: Instant::now(),
            timer_interval: Duration::from_millis(0),
            _tx,
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

    // This will check for an event and return it. If no events are available,
    // then return GlkEvent::None
    pub(crate) fn pop_event(&mut self) -> GlkEvent {
        self.fill_event_queue();

        self.pending.pop_front().unwrap_or(GlkEvent::None)
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
            let now = Instant::now();
            let timeout = (self.last_timer_event + self.timer_interval) - now;
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
