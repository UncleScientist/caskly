use crate::stream::{GlkStreamID, StreamHandler};
use crate::GlkRock;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// A glk window
#[derive(Debug, Default)]
pub struct Window<T: GlkWindow + Default> {
    pub(crate) wintype: WindowType,
    method: Option<WindowSplitMethod>,
    rock: GlkRock,
    parent: Option<Weak<RefCell<Window<T>>>>,
    child1: Option<WindowRef<T>>,
    child2: Option<WindowRef<T>>,
    keywin: KeyWindow,
    #[cfg(test)]
    pub window: Rc<RefCell<T>>,
    #[cfg(not(test))]
    window: Rc<RefCell<T>>,
    stream: GlkStreamID,
}

/// Type of window to create
#[derive(Debug, PartialEq, Clone)]
pub enum GlkWindowType {
    /// A window containing a stream of text
    TextBuffer,

    /// A window containing grid-addressible characters
    TextGrid,

    /// A window that can display colored pixels
    Graphics,

    /// A blank window
    Blank,

    /// A pair window (created internally)
    Pair,
}

/// Interface for a window type; implement this to create a back-end for your
/// window.
pub trait GlkWindow: StreamHandler {
    /// returns the size of the window in its measurement system
    fn get_size(&self) -> GlkWindowSize;

    /// sets the location of the cursor in the window
    fn move_cursor(&mut self, x: u32, y: u32);

    /// clear a window - the way windows get cleared depends on their GlkWindowType
    fn clear(&mut self);
}

/// A GLK window reference
#[derive(Debug, Default)]
pub struct WindowRef<T: GlkWindow + Default> {
    /// the reference to the window
    pub(crate) winref: Rc<RefCell<Window<T>>>,
}

/// The stats from the window that is being closed
#[derive(Debug, Default)]
pub struct StreamResult {
    /// number of characters that were read from this stream
    pub readcount: u32,
    /// number of characters that were written to this stream
    pub writecount: u32,
}

/// The size of a window
#[derive(Debug, Default)]
pub struct GlkWindowSize {
    /// Width of the window in its measurement system (Glk spec section 1.9)
    pub width: u32,

    /// Height of the window in its measurement system (Glk spec section 1.9)
    pub height: u32,
}

#[derive(Debug, Default)]
pub(crate) struct WindowManager<T: GlkWindow + Default> {
    root: WindowRef<T>,
}

impl<T: GlkWindow + Default> WindowManager<T> {
    /// Create a new window
    pub(crate) fn open_window(&self, wintype: WindowType, rock: GlkRock) -> WindowRef<T> {
        assert!(self.root.winref.borrow().wintype == WindowType::Root);
        let root_win = WindowRef {
            winref: Rc::new(RefCell::new(Window {
                wintype,
                rock,
                ..Window::default()
            })),
        };
        root_win
            .winref
            .borrow_mut()
            .parent
            .replace(Rc::downgrade(&self.root.winref));
        self.root
            .winref
            .borrow_mut()
            .child1
            .replace(root_win.make_clone());
        root_win
    }

    pub(crate) fn get_root(&self) -> Option<WindowRef<T>> {
        Some(self.root.winref.borrow().child1.as_ref()?.make_clone())
    }

    fn _dump(&self) {
        self.root._dump(4);
    }
}

impl<T: GlkWindow + Default> WindowRef<T> {
    pub(crate) fn get_winref(&self) -> Rc<RefCell<T>> {
        Rc::clone(&self.winref.borrow().window)
    }

    pub(crate) fn set_stream_id(&self, sid: GlkStreamID) {
        self.winref.borrow_mut().stream = sid;
    }

    fn _dump(&self, indent: usize) {
        println!(
            "{:indent$}{:?} ({}) [parent = {:?}]",
            "",
            self.winref.borrow().wintype,
            self.winref.borrow().rock,
            self.winref.borrow().parent,
        );
        if let Some(child) = &self.winref.borrow().child1 {
            child._dump(indent + 4);
        }

        if let Some(child) = &self.winref.borrow().child2 {
            child._dump(indent + 4);
        }
    }

    // Splitting a singleton Window
    // before:                after:
    //    R                     R
    //    |                     |
    //    W                     P
    //                         / \
    //                        W   N
    //
    // Splitting a child window (window A):
    //     R                     R
    //     |                     |
    //     S                     S
    //    / \                   / \
    //   A   B                 P   B
    //                        / \
    //                       A   N
    /// Split an existing window. Creates a pair window which becomes the
    /// parent of the two windows. The original parent of the split window
    /// becomes the parent of the pair window, and the window being split
    /// becomes the sibling of the new window being created.
    ///
    /// This returns a tuple: the first entry in the tuple contains the pairwin
    /// (if one was created), and the second contains the actual window requested
    /// by the api
    pub(crate) fn split(
        &self,
        method: Option<WindowSplitMethod>,
        wintype: WindowType,
        rock: GlkRock,
    ) -> (WindowRef<T>, WindowRef<T>) {
        assert!(self.winref.borrow().wintype != WindowType::Root);

        let new_win = WindowRef {
            winref: Rc::new(RefCell::new(Window {
                wintype,
                rock,
                ..Window::default()
            })),
        };

        let pair_win = WindowRef {
            winref: Rc::new(RefCell::new(Window {
                wintype: WindowType::Pair,
                method,
                child1: Some(self.make_clone()),
                child2: Some(new_win.make_clone()),
                keywin: KeyWindow::Child2,
                ..Window::default()
            })),
        };

        let old_parent = self
            .winref
            .borrow()
            .parent
            .as_ref()
            .unwrap()
            .upgrade()
            .unwrap();

        let child1 = old_parent.borrow().child1.as_ref().unwrap().winref.clone();
        if Rc::ptr_eq(&self.winref, &child1) {
            old_parent.borrow_mut().child1 = Some(pair_win.make_clone());
        } else if old_parent.borrow().child2.as_ref().is_some() {
            let child2 = old_parent.borrow().child2.as_ref().unwrap().winref.clone();
            assert!(Rc::ptr_eq(&self.winref, &child2));
            old_parent.borrow_mut().child2 = Some(pair_win.make_clone());
        }

        self.winref.borrow_mut().parent = Some(Rc::downgrade(&pair_win.winref));
        new_win.winref.borrow_mut().parent = Some(Rc::downgrade(&pair_win.winref));
        pair_win.winref.borrow_mut().parent = Some(Rc::downgrade(&old_parent));

        (pair_win, new_win)
    }

    fn clean_tree(&mut self) {
        if let Some(child1) = self.winref.borrow_mut().child1.as_mut() {
            child1.clean_tree();
        }

        if let Some(child2) = self.winref.borrow_mut().child2.as_mut() {
            child2.clean_tree();
        }

        self.winref.borrow_mut().child1 = None;
        self.winref.borrow_mut().child2 = None;
    }

    // Closing an existing window (D):
    //
    //     G             G
    //    / \           / \
    //   P   U         C   U
    //  / \
    // C   D
    pub(crate) fn close_window(&self) -> StreamResult {
        let mut parent = self.get_parent().unwrap();
        let grandparent = parent.get_parent().unwrap();

        let sibling = self.get_sibling().unwrap();

        // grandparent's child (parent) is replaced with sibling
        // then close all windows from parent on down

        let is_child1 = if let Some(child1) = grandparent.winref.borrow().child1.as_ref() {
            Rc::ptr_eq(&child1.winref, &parent.winref)
        } else {
            false
        };

        #[cfg(test)]
        {
            let is_child2 = if let Some(child2) = grandparent.winref.borrow().child2.as_ref() {
                Rc::ptr_eq(&child2.winref, &parent.winref)
            } else {
                false
            };

            assert!(is_child1 != is_child2);
        }

        if is_child1 {
            grandparent.winref.borrow_mut().child1 = Some(sibling);
        } else {
            grandparent.winref.borrow_mut().child2 = Some(sibling);
        }

        parent.clean_tree();

        StreamResult::default()
    }

    pub(crate) fn make_clone(&self) -> WindowRef<T> {
        WindowRef {
            winref: Rc::clone(&self.winref),
        }
    }

    /// returns the type of this window
    pub(crate) fn get_type(&self) -> GlkWindowType {
        match self.winref.borrow().wintype {
            WindowType::Blank => GlkWindowType::Blank,
            WindowType::TextBuffer => GlkWindowType::TextBuffer,
            WindowType::TextGrid => GlkWindowType::TextGrid,
            WindowType::Graphics => GlkWindowType::Graphics,
            WindowType::Pair => GlkWindowType::Pair,
            _ => panic!("internal window type only"),
        }
    }

    /// returns the rock value for this window
    pub(crate) fn get_rock(&self) -> GlkRock {
        self.winref.borrow().rock
    }

    /// looks up the parent of this window
    pub fn get_parent(&self) -> Option<WindowRef<T>> {
        Some(WindowRef {
            winref: self.winref.borrow().parent.as_ref()?.upgrade()?,
        })
    }

    /// finds the sibling of the window, NULL if root
    pub fn get_sibling(&self) -> Option<WindowRef<T>> {
        let parent = self.winref.borrow().parent.as_ref()?.upgrade()?;
        if parent.borrow().wintype == WindowType::Root {
            return None;
        }

        if Rc::ptr_eq(&parent.borrow().child1.as_ref()?.winref, &self.winref) {
            Some(parent.borrow().child2.as_ref()?.make_clone())
        } else {
            Some(parent.borrow().child1.as_ref()?.make_clone())
        }
    }

    pub(crate) fn get_size(&self) -> GlkWindowSize {
        self.winref.borrow().window.borrow().get_size()
    }

    pub(crate) fn set_arrangement(&self, method: WindowSplitMethod, keywin: Option<&WindowRef<T>>) {
        if self.winref.borrow().wintype != WindowType::Pair {
            return;
        }

        self.winref.borrow_mut().method = Some(method);
        if let Some(keywin) = keywin {
            if Rc::ptr_eq(
                &self.winref.borrow().child1.as_ref().unwrap().winref,
                &keywin.winref,
            ) {
                self.winref.borrow_mut().keywin = KeyWindow::Child1;
            } else {
                self.winref.borrow_mut().keywin = KeyWindow::Child2;
            }
        }
    }

    /// returns the constraints of the window
    pub(crate) fn get_arrangement(&self) -> Option<(WindowSplitMethod, Option<WindowRef<T>>)> {
        // XXX: this needs to be calculated on the fly, based on how this
        // window was created (e.g. split from another?) and what its parent
        // pair window looks like

        if self.winref.borrow().wintype != WindowType::Pair {
            return None;
        }

        let method = self.winref.borrow().method.as_ref()?.clone();
        let keywin = match self.winref.borrow().keywin {
            KeyWindow::Child1 => Some(self.winref.borrow().child1.as_ref()?.make_clone()),
            KeyWindow::Child2 => Some(self.winref.borrow().child2.as_ref()?.make_clone()),
            KeyWindow::None => None,
        };

        Some((method, keywin))
    }

    pub(crate) fn is_ref(&self, win: &WindowRef<T>) -> bool {
        Rc::ptr_eq(&self.winref, &win.winref)
    }

    pub(crate) fn move_cursor(&self, x: u32, y: u32) {
        if self.winref.borrow().wintype == WindowType::TextGrid {
            self.winref.borrow().window.borrow_mut().move_cursor(x, y);
        }
    }

    pub(crate) fn clear(&self) {
        self.winref.borrow().window.borrow_mut().clear();
    }

    pub(crate) fn get_stream(&self) -> GlkStreamID {
        self.winref.borrow().stream
    }
}

#[derive(Default, Debug)]
enum KeyWindow {
    #[default]
    None,
    Child1,
    Child2,
}

/// Describes how a window should be created when splitting from an existing window
#[derive(Clone, Debug, PartialEq)]
pub struct WindowSplitMethod {
    /// Location of new window in relation to the existing window
    pub position: WindowSplitPosition,

    /// What the new window should look like compared to the existing one
    pub amount: WindowSplitAmount,

    /// Does it have a border?
    pub border: bool,
}

/// Describes where the new window should be placed in relation to the existing window
#[derive(Clone, Debug, PartialEq)]
pub enum WindowSplitPosition {
    /// New window should be above the existing window
    Above,

    /// New window should be below the existing window
    Below,

    /// New window should be to the left of the existing window
    Left,

    /// New window should be to the right of the existing window
    Right,
}

/// How the new window should be sized in relation to the existing window
#[derive(Clone, Debug, PartialEq)]
pub enum WindowSplitAmount {
    /// New window should have a fixed number of lines/columns
    Fixed(i32),

    /// New window should consume a percentage of the existing window
    Proportional(i32),
}

// What kind of window to create
#[derive(Debug, PartialEq, Clone, Default)]
pub(crate) enum WindowType {
    /// A window containing a stream of text
    TextBuffer,

    /// A window containing grid-addressible characters
    TextGrid,

    /// A window that can display colored pixels
    Graphics,

    /// A blank window
    Blank,

    /// A pair window (internal to the library)
    Pair,

    /// Topmost window of the tree
    #[default]
    Root,
}

impl<T: GlkWindow + Default> Window<T> {}

#[cfg(test)]
pub mod testwin {
    use super::*;

    #[derive(Debug)]
    pub struct GlkTestWindow {
        pub width: u32,
        pub height: u32,
        pub cursor_x: u32,
        pub cursor_y: u32,
        pub textdata: String, // output buffer
        pub input_buffer: RefCell<Vec<char>>,
        pub input_cursor: RefCell<usize>,
    }

    impl Default for GlkTestWindow {
        fn default() -> Self {
            Self {
                width: 12,
                height: 32,
                cursor_x: 0,
                cursor_y: 0,
                textdata: String::new(),
                input_buffer: RefCell::new(Vec::new()),
                input_cursor: RefCell::new(0),
            }
        }
    }

    impl StreamHandler for GlkTestWindow {
        fn put_char(&mut self, ch: u8) {
            self.textdata.push(ch as char);
        }

        fn put_char_uni(&mut self, ch: char) {
            self.textdata.push(ch);
        }

        fn put_string(&mut self, s: &str) {
            self.textdata.push_str(s);
        }

        fn put_buffer(&mut self, buf: &[u8]) {
            self.textdata.extend(buf.iter().map(|a| *a as char));
        }

        fn put_buffer_uni(&mut self, buf: &[char]) {
            self.textdata.extend(buf.iter());
        }

        fn get_char(&self) -> Option<u8> {
            let pos = *self.input_cursor.borrow();
            if pos >= self.input_buffer.borrow().len() {
                None
            } else {
                *self.input_cursor.borrow_mut() += 1;
                Some(self.input_buffer.borrow()[pos] as u8)
            }
        }

        fn get_buffer(&self) -> Vec<u8> {
            Vec::new()
        }

        fn get_line(&self) -> Vec<u8> {
            Vec::new()
        }

        fn get_char_uni(&self) -> char {
            '0'
        }

        fn get_buffer_uni(&self) -> Vec<char> {
            Vec::new()
        }

        fn get_line_uni(&self) -> Vec<char> {
            Vec::new()
        }
    }

    impl super::GlkWindow for GlkTestWindow {
        fn get_size(&self) -> GlkWindowSize {
            GlkWindowSize {
                width: self.width,
                height: self.height,
            }
        }

        fn move_cursor(&mut self, x: u32, y: u32) {
            self.cursor_x = x;
            self.cursor_y = y;
        }

        fn clear(&mut self) {
            self.cursor_x = 0;
            self.cursor_y = 0;
        }
    }

    impl GlkTestWindow {
        pub fn set_input_buffer(&mut self, s: &str) {
            self.input_buffer = RefCell::new(Vec::from_iter(s.chars()));
            self.input_cursor = RefCell::new(0);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use testwin::*;

    #[test]
    fn can_create_window() {
        let winsys = WindowManager::<GlkTestWindow>::default();

        let win = winsys.open_window(WindowType::TextBuffer, 32);
        assert_eq!(win.get_type(), GlkWindowType::TextBuffer);
    }

    #[test]
    fn can_split_window() {
        let winsys = WindowManager::<GlkTestWindow>::default();
        let root_win = winsys.open_window(WindowType::TextBuffer, 32);

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Fixed(3),
            border: false,
        };

        let (_, split) = root_win.split(Some(method), WindowType::TextBuffer, 65);

        let parent = split.get_parent().unwrap();
        assert_eq!(parent.get_rock(), 0);

        let sibling = split.get_sibling().unwrap();
        assert_eq!(sibling.get_rock(), 32);

        let sibling = root_win.get_sibling().unwrap();
        assert_eq!(sibling.get_rock(), 65);
    }

    #[test]
    fn can_split_multiple_times() {
        let winsys = WindowManager::<GlkTestWindow>::default();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(50),
            border: false,
        };

        let window_a = winsys.open_window(WindowType::TextBuffer, 32);
        let (_, window_b) = window_a.split(Some(method.clone()), WindowType::TextBuffer, 33);
        let (_, _) = window_a.split(Some(method.clone()), WindowType::TextBuffer, 34);

        let sibling = window_a.get_sibling().unwrap();
        assert_eq!(sibling.get_rock(), 34);

        let sibling = window_b.get_sibling().unwrap();
        assert_eq!(sibling.get_type(), GlkWindowType::Pair);
    }

    #[test]
    fn can_destroy_window() {
        let winsys = WindowManager::<GlkTestWindow>::default();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(50),
            border: false,
        };

        let window_a = winsys.open_window(WindowType::TextBuffer, 32);
        let (_, _) = window_a.split(Some(method.clone()), WindowType::TextBuffer, 33);

        let (_, window_c) = window_a.split(Some(method.clone()), WindowType::TextBuffer, 34);
        let (_, window_d) = window_c.split(Some(method.clone()), WindowType::TextBuffer, 35);

        let parent = window_d.get_parent().unwrap();
        let sibling = parent.get_sibling().unwrap();

        assert_eq!(sibling.get_rock(), 32);

        println!("---\nbefore:");
        winsys._dump();
        window_d.close_window();
        println!("\n\n---\nafter:");
        winsys._dump();

        assert_eq!(window_a.get_sibling().unwrap().get_rock(), 34);
    }

    #[test]
    fn can_retrieve_window_size() {
        let winsys = WindowManager::<GlkTestWindow>::default();
        let root_window = winsys.open_window(WindowType::TextBuffer, 32);
        let size = root_window.get_size();
        assert_eq!(size.width, 12);
        assert_eq!(size.height, 32);
    }

    #[test]
    fn can_get_window_constraints() {
        let winsys = WindowManager::<GlkTestWindow>::default();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(20),
            border: false,
        };

        let window_a = winsys.open_window(WindowType::TextBuffer, 32);
        let (_, window_b) = window_a.split(Some(method.clone()), WindowType::TextBuffer, 33);

        let (pair_method, _) = window_b.get_parent().unwrap().get_arrangement().unwrap();
        assert_eq!(method.position, pair_method.position);
        assert_eq!(method.amount, pair_method.amount);
        assert_eq!(method.border, pair_method.border);
    }

    #[test]
    fn default_to_child2_for_key_window() {
        let winsys = WindowManager::<GlkTestWindow>::default();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(20),
            border: false,
        };

        let window_a = winsys.open_window(WindowType::TextBuffer, 32);
        let (_, window_b) = window_a.split(Some(method.clone()), WindowType::TextBuffer, 33);

        let (_, keywin) = window_b.get_parent().unwrap().get_arrangement().unwrap();
        assert!(Rc::ptr_eq(&keywin.unwrap().winref, &window_b.winref));
    }

    #[test]
    fn can_change_key_window_to_child1() {
        let winsys = WindowManager::<GlkTestWindow>::default();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(20),
            border: false,
        };

        let window_a = winsys.open_window(WindowType::TextBuffer, 32);
        let (_, _) = window_a.split(Some(method.clone()), WindowType::TextBuffer, 33);

        let parent = window_a.get_parent().unwrap();
        parent.set_arrangement(method.clone(), Some(&window_a));

        let (_, keywin) = parent.get_arrangement().unwrap();
        assert!(Rc::ptr_eq(&keywin.unwrap().winref, &window_a.winref));
    }

    #[test]
    fn cannot_get_arrangement_for_non_pair_window() {
        let winsys = WindowManager::<GlkTestWindow>::default();
        let window_a = winsys.open_window(WindowType::TextBuffer, 32);
        assert!(window_a.get_arrangement().is_none());
    }

    #[test]
    fn can_move_cursor_in_a_window() {
        let winsys = WindowManager::<GlkTestWindow>::default();
        let window_a = winsys.open_window(WindowType::TextBuffer, 32);

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(20),
            border: false,
        };
        let (_, window_b) = window_a.split(Some(method), WindowType::TextGrid, 55);

        // text buffers do not use move_cursor
        window_a.move_cursor(4, 4);
        assert_eq!(window_a.winref.borrow().window.borrow().cursor_x, 0);

        // text grid windows DO move the cursor
        window_b.move_cursor(4, 4);
        assert_eq!(window_b.winref.borrow().window.borrow().cursor_x, 4);
    }

    #[test]
    fn can_clear_window() {
        let winsys = WindowManager::<GlkTestWindow>::default();
        let window_a = winsys.open_window(WindowType::TextGrid, 32);
        window_a.move_cursor(5, 5);
        assert_eq!(window_a.winref.borrow().window.borrow().cursor_x, 5);
        window_a.clear();
        assert_eq!(window_a.winref.borrow().window.borrow().cursor_x, 0);
    }

    #[test]
    fn can_set_input_buffer_in_test_window() {
        let winsys = WindowManager::<GlkTestWindow>::default();
        let window_a = winsys.open_window(WindowType::TextGrid, 32);
        window_a
            .winref
            .borrow()
            .window
            .borrow_mut()
            .set_input_buffer("test buffer");
        assert_eq!(
            window_a
                .winref
                .borrow()
                .window
                .borrow()
                .input_buffer
                .borrow()
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            "test buffer".chars().collect::<Vec<_>>()
        );
    }

    /*
    #[test]
    fn can_put_character_in_window() {
        let winsys = WindowManager::<GlkTestWindow>::default();
        let win = winsys.open_window(WindowType::TextGrid, 32);
        win.put_char(b'a');
        assert_eq!(win.winref.borrow().window.textdata, "a");
    }
    */
}
