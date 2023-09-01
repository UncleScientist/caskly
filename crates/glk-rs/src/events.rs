use std::collections::VecDeque;

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

    /// A sound finished playing
    SoundNotify {
        /// the Sound Resource (from the blorb file) that finished playing
        resource_id: u32,
        /// The user's notification value
        notify: u32,
    },

    /// A hyperlink was clicked/selected
    Hyperlink {
        /// The window generating the hyperlink event
        win: GlkWindowID,
        /// The user's linkval for the selected link
        linkval: u32,
    },

    /// A volume change was completed
    VolumeNotify {
        /// The user's notification value
        notify: u32,
    },
}

#[derive(Default)]
pub(crate) struct EventManager {
    pending: VecDeque<GlkEvent>,
}

impl EventManager {
    // pub(crate) fn has_an_event() -> bool { !self.pending.is_empty() }

    pub(crate) fn pop_event(&mut self) -> GlkEvent {
        self.pending.pop_front().unwrap_or(GlkEvent::None)
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
