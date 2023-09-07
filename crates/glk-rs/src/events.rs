use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::Duration,
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
    timer_fired: bool,
    timer_interval: u64,
    timer_thread: Option<JoinHandle<()>>,
    timer_update_channel: Option<Sender<u64>>,
    tx: Sender<GlkEvent>,
    rx: Receiver<GlkEvent>,
}

impl Default for EventManager {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            pending: VecDeque::new(),
            timer_fired: false,
            timer_interval: 0,
            timer_thread: None,
            timer_update_channel: None,
            tx,
            rx,
        }
    }
}

impl EventManager {
    fn fill_event_queue(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                GlkEvent::Timer => self.timer_fired = true,
                other => self.pending.push_back(other),
            }
        }
    }

    // This will check for an event and return it. If no events are available,
    // then return GlkEvent::None
    pub(crate) fn pop_event(&mut self) -> GlkEvent {
        self.fill_event_queue();

        if let Some(event) = self.pending.pop_front() {
            return event;
        }

        if self.timer_fired {
            self.timer_fired = false;
            if let Some(channel) = self.timer_update_channel.as_ref() {
                let _ = channel.send(self.timer_interval);
            }
            return GlkEvent::Timer;
        }

        GlkEvent::None
    }

    // This will block until an event is available, and then return it. Should never
    // return GlkEvent::None
    pub(crate) fn block_until_event(&mut self) -> GlkEvent {
        self.fill_event_queue();

        let next_event = match self.pop_event() {
            GlkEvent::None => {
                if let Some(channel) = self.timer_update_channel.as_ref() {
                    let _ = channel.send(self.timer_interval);
                }
                self.rx
                    .recv()
                    .expect("library bug: should get an event here")
            }
            event => event,
        };

        next_event
    }

    pub(crate) fn set_timer(&mut self, ms: u32) {
        self.timer_interval = ms as u64;
        if self.timer_thread.is_some() {
            let _ = self
                .timer_update_channel
                .as_ref()
                .expect("library bug: channel is missing from timer")
                .send(ms as u64);
            if ms == 0 {
                let tt = self
                    .timer_thread
                    .take()
                    .expect("rust bug: rust returned true from is_some()");
                let _ = tt.join();
                self.timer_update_channel = None;
            }
        } else {
            let (tx, rx) = mpsc::channel();
            self.timer_update_channel = Some(tx);
            let event_queue = self.tx.clone();
            self.timer_thread = Some(thread::spawn(move || {
                timer_thread(event_queue, rx, ms as u64)
            }));
        }
    }
}

fn timer_thread(fire: Sender<GlkEvent>, next: Receiver<u64>, ms: u64) {
    let mut current_interval = Duration::from_millis(ms);
    loop {
        if current_interval.is_zero() {
            break;
        }

        match next.recv_timeout(current_interval) {
            Ok(new_interval) => current_interval = Duration::from_millis(new_interval),
            Err(RecvTimeoutError::Timeout) => {
                let _ = fire.send(GlkEvent::Timer);
                if let Ok(new_time) = next.recv() {
                    current_interval = Duration::from_millis(new_time);
                } else {
                    break;
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
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
