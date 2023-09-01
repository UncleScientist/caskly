use crate::{events::GlkEvent, windows::GlkWindow};

use super::Glk;

impl<T: GlkWindow + Default> Glk<T> {
    /// Block until event arrives
    pub fn select_poll(&mut self) -> GlkEvent {
        self.event_mgr.pop_event()
    }
}

#[cfg(test)]
mod test {
    use crate::windows::testwin::GlkTestWindow;

    use super::*;

    #[test]
    fn can_check_for_events() {
        let mut glk = Glk::<GlkTestWindow>::new();

        assert_eq!(glk.select_poll(), GlkEvent::None);
    }
}
