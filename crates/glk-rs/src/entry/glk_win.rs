use std::{cell::RefCell, rc::Rc};

use crate::{
    prelude::GlkRock,
    stream::{GlkStreamID, GlkStreamResult},
    windows::{
        GlkWindow, GlkWindowID, GlkWindowSize, GlkWindowType, WindowRef, WindowSplitMethod,
        WindowType,
    },
    Glk, GlkFileMode,
};

impl<T: GlkWindow + Default> Glk<T> {
    /*
     * Glk Spec Section 3.2 - Window Opening, Closing, and Constraints
     */

    /// create a new window
    pub fn window_open(
        &mut self,
        parent: Option<GlkWindowID>,
        wintype: GlkWindowType,
        method: Option<WindowSplitMethod>,
        rock: GlkRock,
    ) -> Option<GlkWindowID> {
        let wintype = match wintype {
            GlkWindowType::Blank => WindowType::Blank,
            GlkWindowType::TextBuffer => WindowType::TextBuffer,
            GlkWindowType::TextGrid => WindowType::TextGrid,
            GlkWindowType::Graphics => WindowType::Graphics,
            GlkWindowType::Pair => return None,
        };

        let new_win = if let Some(parent) = parent {
            self.win_mgr.split(parent, method, wintype, rock)
        } else {
            self.win_mgr.open_window(wintype, rock)
        }?;

        let win = Rc::new(RefCell::new(self.win_mgr.get_window(new_win)?));
        let stream_id = self.stream_mgr.new_stream(win, GlkFileMode::Write);
        self.win_mgr.set_stream_id(new_win, stream_id)?;

        Some(new_win)
    }

    /// close the given window and all of its children
    pub fn window_close(&mut self, win: GlkWindowID) -> Option<GlkStreamResult> {
        let winref = self.win_mgr.get_ref(win)?;
        let stream = winref.get_stream();

        self.win_mgr.close(win)?;
        self.stream_mgr.close(stream)
    }

    /*
     * Glk Spec Section 3.3 - Changing Window Constraints
     */

    /// get the actual size of the window, in its measurement system
    pub fn window_get_size(&self, win: &WindowRef<T>) -> GlkWindowSize {
        win.get_size()
    }

    /// Get the size of the window in its measurement system (Glk Spec section 1.9)
    pub fn window_set_arrangement(
        &self,
        win: &WindowRef<T>,
        method: WindowSplitMethod,
        keywin: Option<&WindowRef<T>>,
    ) {
        win.set_arrangement(method, keywin);
    }

    /// returns the constraints of the window
    pub fn window_get_arrangement(
        &self,
        win: GlkWindowID,
    ) -> (Option<WindowSplitMethod>, Option<GlkWindowID>) {
        if let Some(winref) = self.win_mgr.get_ref(win) {
            if let Some((method, keywin)) = winref.get_arrangement() {
                let keywin_id = keywin.map(|k| k.id());
                return (Some(method), keywin_id);
            }
        }

        (None, None)
    }

    /*
     * Glk Spec Section 3.5.4 - Text Grid Windows
     */

    /// Move the cursor in a text grid window (all other window types ignore this API)
    pub fn window_move_cursor(&self, win: &WindowRef<T>, xpos: u32, ypos: u32) {
        win.move_cursor(xpos, ypos);
    }

    /*
     * Glk Spec Section 3.6 - Echo Streams
     */

    /// set the echo stream of a window
    pub fn window_set_echo_stream(&mut self, win: GlkWindowID, stream: Option<GlkStreamID>) {
        self.win_mgr.set_echo_stream(win, stream);
    }

    /// get the echo stream of a window
    pub fn window_get_echo_stream(&self, win: GlkWindowID) -> Option<GlkStreamID> {
        self.win_mgr.get_echo_stream(win)
    }

    /*
     * Glk Spec Section 3.7 - Other Window Functions
     */

    /// iterate through all the windows
    pub fn window_iterate(&self) -> std::vec::IntoIter<GlkWindowID> {
        // should we be doing this with Iter<&WindowRef<T>> instead?
        self.win_mgr.get_iter()
    }

    /// get the rock value for a given window
    pub fn window_get_rock(&self, win: GlkWindowID) -> Option<GlkRock> {
        let win = self.win_mgr.get_ref(win)?;
        Some(win.get_rock())
    }

    /// get the type of the window
    pub fn window_get_type(&self, win: GlkWindowID) -> Option<GlkWindowType> {
        let win = self.win_mgr.get_ref(win)?;
        Some(win.get_type())
    }

    /// get the parent for this window
    pub fn window_get_parent(&self, win: GlkWindowID) -> Option<GlkWindowID> {
        let win = self.win_mgr.get_ref(win)?;
        let parent = win.get_parent()?;
        if parent.winref.borrow().wintype == WindowType::Root {
            None
        } else {
            Some(parent.id())
        }
    }

    /// get the sibling of this window
    pub fn window_get_sibling(&self, win: GlkWindowID) -> Option<GlkWindowID> {
        let win = self.win_mgr.get_ref(win)?;
        Some(win.get_sibling()?.id())
    }

    /// gets the root window - if there are no windows, returns None
    pub fn window_get_root(&self) -> Option<GlkWindowID> {
        self.win_mgr.get_root()
    }

    /// clears the window
    pub fn window_clear(&self, win: GlkWindowID) {
        if let Some(win) = self.win_mgr.get_ref(win) {
            win.clear();
        }
    }

    /// get the stream associated with a window
    pub fn window_get_stream(&self, win: GlkWindowID) -> Option<GlkStreamID> {
        let win = self.win_mgr.get_ref(win)?;
        Some(win.get_stream())
    }

    /* TEST ONLY FUNCTIONS */
    #[cfg(test)]
    pub(crate) fn t_get_winref(&self, win: GlkWindowID) -> WindowRef<T> {
        self.win_mgr.get_ref(win).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::windows::{testwin::GlkTestWindow, WindowSplitAmount, WindowSplitPosition};

    #[test]
    fn can_create_a_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk.window_open(None, GlkWindowType::TextBuffer, None, 73);
        assert!(win.is_some());
    }

    #[test]
    fn can_create_a_split_window() {
        let mut glk = Glk::<GlkTestWindow>::new();

        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let win2 = glk.window_open(
            Some(win),
            GlkWindowType::TextGrid,
            Some(WindowSplitMethod {
                position: WindowSplitPosition::Above,
                amount: WindowSplitAmount::Proportional(40),
                border: false,
            }),
            84,
        );
        assert!(win2.is_some());
    }

    #[test]
    fn can_retrieve_window_information() {
        let mut glk = Glk::<GlkTestWindow>::new();

        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        assert_eq!(glk.window_get_rock(win).unwrap(), 73);
        assert_eq!(glk.window_get_type(win).unwrap(), GlkWindowType::TextBuffer);
    }

    #[test]
    #[should_panic]
    fn must_use_existing_window_for_splits() {
        let mut glk = Glk::<GlkTestWindow>::new();

        glk.window_open(None, GlkWindowType::TextBuffer, None, 73);
        glk.window_open(None, GlkWindowType::TextBuffer, None, 73);
    }

    #[test]
    fn can_iterate_windows() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let win2 = glk
            .window_open(
                Some(win1),
                GlkWindowType::TextGrid,
                Some(WindowSplitMethod {
                    position: WindowSplitPosition::Above,
                    amount: WindowSplitAmount::Proportional(40),
                    border: false,
                }),
                84,
            )
            .unwrap();
        let win3 = glk
            .window_open(
                Some(win2),
                GlkWindowType::TextGrid,
                Some(WindowSplitMethod {
                    position: WindowSplitPosition::Below,
                    amount: WindowSplitAmount::Fixed(3),
                    border: false,
                }),
                95,
            )
            .unwrap();

        // pair1, pair2, win1, win2, win3
        let mut found = [false, false, false, false, false];
        let i = glk.window_iterate();
        let mut count = 0;
        let mut found_pair = None;
        for win in i {
            count += 1;
            if win == win1 {
                found[2] = true;
            } else if win == win2 {
                found[3] = true;
            } else if win == win3 {
                found[4] = true;
            } else if found_pair.is_none() {
                found_pair = Some(win);
                found[0] = true;
            } else if let Some(f) = found_pair {
                if f != win {
                    found[1] = true;
                }
            }
        }
        assert_eq!(count, 5);
        assert_eq!([true, true, true, true, true], found);
    }

    #[test]
    fn can_get_parent_of_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        assert!(glk.window_get_parent(win1).is_none());
        let win2 = glk
            .window_open(
                Some(win1),
                GlkWindowType::TextGrid,
                Some(WindowSplitMethod {
                    position: WindowSplitPosition::Above,
                    amount: WindowSplitAmount::Proportional(40),
                    border: false,
                }),
                84,
            )
            .unwrap();
        let parent1 = glk.window_get_parent(win2).unwrap();
        assert_eq!(glk.window_get_type(parent1).unwrap(), GlkWindowType::Pair);
    }

    #[test]
    fn can_get_sibling_of_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        assert!(glk.window_get_sibling(win1).is_none());

        let win2 = glk
            .window_open(
                Some(win1),
                GlkWindowType::TextGrid,
                Some(WindowSplitMethod {
                    position: WindowSplitPosition::Above,
                    amount: WindowSplitAmount::Proportional(40),
                    border: false,
                }),
                84,
            )
            .unwrap();
        let sibling = glk.window_get_sibling(win2).unwrap();
        assert_eq!(sibling, win1);
    }

    #[test]
    fn can_get_root_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        assert!(glk.window_get_root().is_none());
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        assert_eq!(glk.window_get_root().unwrap(), win1);
    }

    #[test]
    fn can_put_byte_style_char_into_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(win).unwrap();
        glk.put_char_stream(stream, b'x');
        let winref = glk.t_get_winref(win);
        assert_eq!(winref.winref.borrow().window.borrow().textdata, "x");
    }

    #[test]
    fn can_write_to_two_different_windows() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        assert!(glk.window_get_parent(win1).is_none());
        let win2 = glk
            .window_open(
                Some(win1),
                GlkWindowType::TextGrid,
                Some(WindowSplitMethod {
                    position: WindowSplitPosition::Above,
                    amount: WindowSplitAmount::Proportional(40),
                    border: false,
                }),
                84,
            )
            .unwrap();

        let stream1 = glk.window_get_stream(win1).unwrap();
        let stream2 = glk.window_get_stream(win2).unwrap();

        glk.put_char_stream(stream1, b'A');
        glk.put_char_stream(stream2, b'B');

        let win1 = glk.t_get_winref(win1);
        let win2 = glk.t_get_winref(win2);
        assert_eq!(win1.winref.borrow().window.borrow().textdata, "A");
        assert_eq!(win2.winref.borrow().window.borrow().textdata, "B");
    }

    #[test]
    fn can_put_string_into_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(win).unwrap();
        glk.put_string_stream(stream, "hello, world!");
        let win = glk.t_get_winref(win);
        assert_eq!(
            win.winref.borrow().window.borrow().textdata,
            "hello, world!"
        );
    }

    #[test]
    fn can_put_buffer_into_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(win).unwrap();
        glk.put_buffer_stream(stream, &[b'0', b'1', b'2', b'3']);
        let win = glk.t_get_winref(win);
        assert_eq!(win.winref.borrow().window.borrow().textdata, "0123");
    }

    #[test]
    fn can_put_unicode_char_into_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(win).unwrap();
        glk.put_char_stream_uni(stream, 'q');
        let win = glk.t_get_winref(win);
        assert_eq!(win.winref.borrow().window.borrow().textdata, "q");
    }

    #[test]
    fn can_put_unicode_buf_into_window() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(win).unwrap();
        glk.put_buffer_stream_uni(stream, &['q', 'r', 's', 't', 'u', 'v']);
        let win = glk.t_get_winref(win);
        assert_eq!(win.winref.borrow().window.borrow().textdata, "qrstuv");
    }

    #[test]
    fn can_change_default_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let stream = glk.window_get_stream(win).unwrap();
        glk.stream_set_current(stream);
        assert!(glk.stream_get_current().is_some());
        assert_eq!(glk.stream_get_current(), Some(stream));
    }

    #[test]
    fn can_write_to_default_stream() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let win2 = glk
            .window_open(
                Some(win1),
                GlkWindowType::TextGrid,
                Some(WindowSplitMethod {
                    position: WindowSplitPosition::Above,
                    amount: WindowSplitAmount::Proportional(40),
                    border: false,
                }),
                84,
            )
            .unwrap();

        let stream1 = glk.window_get_stream(win1).unwrap();
        let stream2 = glk.window_get_stream(win2).unwrap();

        glk.stream_set_current(stream1);
        glk.put_char(b'A');
        glk.put_string("bove");
        glk.put_buffer(&[b' ', b't', b'h', b'e']);
        glk.put_char_uni(' ');
        glk.put_buffer_uni(&['s', 'k', 'y']);

        glk.stream_set_current(stream2);
        glk.put_char(b'B');
        glk.put_string("elow");
        glk.put_buffer(&[b' ', b'g', b'r', b'o', b'u', b'n', b'd']);
        glk.put_char_uni('.');
        glk.put_buffer_uni(&[' ', 'L', 'o', 'o', 'k', '!']);

        let win1 = glk.t_get_winref(win1);
        assert_eq!(
            win1.winref.borrow().window.borrow().textdata,
            "Above the sky"
        );
        let win2 = glk.t_get_winref(win2);
        assert_eq!(
            win2.winref.borrow().window.borrow().textdata,
            "Below ground. Look!"
        );
    }

    #[test]
    fn can_count_chars_in_output() {
        let mut glk = Glk::<GlkTestWindow>::new();
        let win1 = glk
            .window_open(None, GlkWindowType::TextBuffer, None, 73)
            .unwrap();
        let win2 = glk
            .window_open(
                Some(win1),
                GlkWindowType::TextGrid,
                Some(WindowSplitMethod {
                    position: WindowSplitPosition::Above,
                    amount: WindowSplitAmount::Proportional(40),
                    border: false,
                }),
                84,
            )
            .unwrap();
        let stream2 = glk.window_get_stream(win2).unwrap();

        glk.put_char_stream(stream2, b'0');
        let stream_results = glk.window_close(win2).unwrap();
        assert_eq!(stream_results.read_count, 0);
        assert_eq!(stream_results.write_count, 1);
    }
}
