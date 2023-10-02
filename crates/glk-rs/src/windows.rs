use crate::entry::{GlkMessage, GlkResult};
use crate::events::{GlkEvent, LineInput};
use crate::prelude::GlkRock;
use crate::stream::{GlkStreamHandler, GlkStreamID, WriteResponse};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::mpsc::{Receiver, Sender};

/// An opaque type for windows
pub type GlkWindowID = u32;

/// A glk window
#[derive(Default)]
pub struct Window<T: GlkWindow + Default> {
    pub(crate) wintype: WindowType,
    method: Option<WindowSplitMethod>,
    rock: GlkRock,
    this_id: GlkWindowID,
    parent: Option<Weak<RefCell<Window<T>>>>,
    child1: Option<WindowRef<T>>,
    child2: Option<WindowRef<T>>,
    keywin: KeyWindow,
    #[cfg(test)]
    pub window: Rc<RefCell<T>>,
    #[cfg(not(test))]
    window: Rc<RefCell<T>>,
    stream: GlkStreamID,
    echo_stream: Option<GlkStreamID>,
    command: Option<Sender<GlkMessage>>,
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
pub trait GlkWindow {
    /// Build a new GlkWindow system
    fn new(request: Receiver<GlkMessage>, result: Sender<GlkResult>) -> Self;

    /// Primary run loop for stdio or window system
    fn run(&mut self);

    /// set up a window with specific parameters
    fn init(&mut self, winid: GlkWindowID);

    /// returns the size of the window in its measurement system
    fn get_size(&self) -> GlkWindowSize;

    /// sets the location of the cursor in the window
    fn move_cursor(&mut self, x: u32, y: u32);

    /// clear a window - the way windows get cleared depends on their GlkWindowType
    fn clear(&mut self);

    /// read a line from a window and transmit it to the event queue - must run separate thread
    fn get_line(&mut self, event: LineInput, initlen: usize, tx: Sender<GlkEvent>);

    /// write a byte to a window
    fn write_char(&mut self, ch: u8) -> usize;

    /// write a string to a window
    fn write_string(&mut self, s: &str) -> usize;

    /// write an array of bytes to a window
    fn write_buffer(&mut self, buf: &[u8]) -> usize;

    /// write a unicode character to a window
    fn write_char_uni(&mut self, ch: char) -> usize;

    /// write an array of unicode characters to a window
    fn write_buffer_uni(&mut self, buf: &[char]) -> usize;
}

/// A GLK window reference
#[derive(Default)]
pub struct WindowRef<T: GlkWindow + Default> {
    /// the reference to the window
    pub(crate) winref: Rc<RefCell<Window<T>>>,
}

impl<T: GlkWindow + Default> GlkStreamHandler for WindowRef<T> {
    fn get_echo_stream(&self) -> Option<GlkStreamID> {
        self.winref.borrow().echo_stream
    }

    fn put_char(&mut self, ch: u8) -> WriteResponse {
        let mut message = String::new();
        message.push(ch as char);
        self.write_string(&message)
    }

    fn put_string(&mut self, s: &str) -> WriteResponse {
        self.write_string(s)
    }

    fn put_buffer(&mut self, buf: &[u8]) -> WriteResponse {
        let message = buf.iter().map(|byte| *byte as char).collect::<String>();
        self.write_string(&message)
    }

    fn put_char_uni(&mut self, ch: char) -> WriteResponse {
        let mut message = String::new();
        message.push(ch);
        self.write_string(&message)
    }

    fn put_buffer_uni(&mut self, buf: &[char]) -> WriteResponse {
        let message = buf.iter().collect::<String>();
        self.write_string(&message)
    }

    fn get_char(&mut self) -> Option<u8> {
        panic!("Library Bug: Should not call this function");
    }

    fn get_buffer(&mut self, _maxlen: Option<usize>) -> Vec<u8> {
        panic!("Library Bug: Should not call this function");
    }

    fn get_line(&mut self, _maxlen: Option<usize>) -> Vec<u8> {
        panic!("Library Bug: Should not call this function");
    }

    fn get_char_uni(&mut self) -> Option<char> {
        panic!("Library Bug: Should not call this function");
    }

    fn get_buffer_uni(&mut self, _maxlen: Option<usize>) -> String {
        panic!("Library Bug: Should not call this function");
    }

    fn get_line_uni(&mut self, _maxlen: Option<usize>) -> String {
        panic!("Library Bug: Should not call this function");
    }

    fn get_position(&self) -> u32 {
        // Glk spec section 5.4, window streams always return 0 for get_position()
        0
    }

    fn set_position(&mut self, _pos: i32, _seekmode: crate::GlkSeekMode) -> Option<()> {
        // Glk Spec section 5.4, window streams ignore calls to set_position
        Some(())
    }

    fn get_data(&self) -> Vec<u8> {
        panic!("Library Bug: Should not call this function");
    }

    fn close(&mut self) {
        // no-op
    }

    fn is_window_stream(&self) -> bool {
        true
    }

    fn is_memory_stream(&self) -> bool {
        false
    }
}

/// The size of a window
#[derive(Debug, Default)]
pub struct GlkWindowSize {
    /// Width of the window in its measurement system (Glk spec section 1.9)
    pub width: u32,

    /// Height of the window in its measurement system (Glk spec section 1.9)
    pub height: u32,
}

#[derive(Default)]
pub(crate) struct WindowManager<T: GlkWindow + Default> {
    root: Option<GlkWindowID>,
    windows: HashMap<GlkWindowID, WindowRef<T>>,
    val: GlkWindowID,
}

impl<T: GlkWindow + Default> WindowManager<T> {
    /// Create the first window in the hierarchy
    pub(crate) fn open_window(
        &mut self,
        wintype: WindowType,
        command: Sender<GlkMessage>,
        rock: GlkRock,
    ) -> Option<GlkWindowID> {
        if self.root.is_none() {
            assert!(self.windows.is_empty());
            let root_win = WindowRef {
                winref: Rc::new(RefCell::new(Window::<T> {
                    wintype: WindowType::Root,
                    command: Some(command.clone()),
                    ..Window::default()
                })),
            };
            root_win.winref.borrow().window.borrow_mut().init(self.val);
            self.root = Some(0);
            self.windows.insert(0, root_win);
            self.val += 1;
        }

        assert_eq!(self.windows.len(), 1);

        let root_win = self.windows.get(&0).unwrap();
        let main_win = WindowRef {
            winref: Rc::new(RefCell::new(Window {
                wintype,
                rock,
                this_id: self.val,
                parent: Some(Rc::downgrade(&root_win.winref)),
                command: Some(command),
                ..Window::default()
            })),
        };

        main_win.winref.borrow().window.borrow_mut().init(self.val);
        root_win.winref.borrow_mut().child1 = Some(main_win.make_clone());

        self.windows.insert(self.val, main_win);

        self.val += 1;

        Some(self.val - 1)
    }

    pub(crate) fn get_root(&self) -> Option<GlkWindowID> {
        let win = self.windows.get(&self.root?)?;
        Some(win.winref.borrow().child1.as_ref()?.id())
    }

    pub(crate) fn get_ref(&self, win: GlkWindowID) -> Option<WindowRef<T>> {
        Some(self.windows.get(&win)?.make_clone())
    }

    pub(crate) fn get_window(&self, win: GlkWindowID) -> Option<WindowRef<T>> {
        Some(self.windows.get(&win)?.make_clone())
    }

    pub(crate) fn get_iter(&self) -> std::vec::IntoIter<GlkWindowID> {
        self.windows
            .keys()
            .copied()
            .filter(|x| *x != 0)
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub(crate) fn set_stream_id(&self, win: GlkWindowID, stream: GlkStreamID) -> Option<()> {
        self.windows.get(&win)?.set_stream_id(stream);
        Some(())
    }

    pub(crate) fn set_echo_stream(&self, win: GlkWindowID, stream: Option<GlkStreamID>) {
        if let Some(window) = self.windows.get(&win) {
            window.winref.borrow_mut().echo_stream = stream;
        }
    }

    pub(crate) fn get_echo_stream(&self, win: GlkWindowID) -> Option<GlkStreamID> {
        self.windows.get(&win)?.winref.borrow_mut().echo_stream
    }

    pub(crate) fn split(
        &mut self,
        parent: GlkWindowID,
        method: Option<WindowSplitMethod>,
        wintype: WindowType,
        command: Sender<GlkMessage>,
        rock: GlkRock,
    ) -> Option<GlkWindowID> {
        let parentwin = self.windows.get(&parent)?;

        let (pairwin, newwin) = parentwin.split(method, wintype, command, rock);

        pairwin.winref.borrow_mut().this_id = self.val;
        pairwin.winref.borrow().window.borrow_mut().init(self.val);
        self.windows.insert(self.val, pairwin);
        self.val += 1;

        newwin.winref.borrow_mut().this_id = self.val;
        newwin.winref.borrow().window.borrow_mut().init(self.val);
        self.windows.insert(self.val, newwin);
        self.val += 1;

        Some(self.val - 1)
    }

    pub(crate) fn close(&mut self, win: GlkWindowID) -> Option<()> {
        let winref = self.windows.remove(&win)?;
        winref.close_window();
        Some(())
    }

    fn _dump(&self) {
        if let Some(root) = self.root {
            let rootwin = self.windows.get(&root).unwrap();
            rootwin._dump(4);
        }
    }
}

impl<T: GlkWindow + Default> WindowRef<T> {
    pub(crate) fn send_message(&self, message: GlkMessage) {
        let _ = self.winref.borrow().command.as_ref().unwrap().send(message);
    }

    fn write_string(&self, s: &str) -> WriteResponse {
        let _ = self.send_message(GlkMessage::Write {
            winid: self.winref.borrow().this_id,
            message: s.to_string(),
        });
        WriteResponse {
            len: 0,
            wait_needed: true,
        }
    }

    pub(crate) fn get_line(&self, input: LineInput, initlen: usize, tx: Sender<GlkEvent>) {
        self.winref
            .borrow()
            .window
            .borrow_mut()
            .get_line(input, initlen, tx);
    }

    pub(crate) fn remove_echo_stream_if_matches(&mut self, stream: GlkStreamID) {
        if self.winref.borrow().echo_stream == Some(stream) {
            self.winref.borrow_mut().echo_stream = None;
        }
    }

    pub(crate) fn id(&self) -> GlkWindowID {
        self.winref.borrow().this_id
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
        command: Sender<GlkMessage>,
        rock: GlkRock,
    ) -> (WindowRef<T>, WindowRef<T>) {
        assert!(self.winref.borrow().wintype != WindowType::Root);

        let new_win = WindowRef {
            winref: Rc::new(RefCell::new(Window {
                wintype,
                rock,
                command: Some(command.clone()),
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
                command: Some(command),
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
    pub(crate) fn close_window(&self) {
        let mut parent = self.get_parent().unwrap();

        if let Some(grandparent) = parent.get_parent() {
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
        }

        parent.clean_tree();
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
        pub winid: GlkWindowID,
        pub width: u32,
        pub height: u32,
        pub cursor_x: u32,
        pub cursor_y: u32,
        pub textdata: String, // output buffer
        pub input_buffer: RefCell<Vec<char>>,
        pub input_cursor: RefCell<usize>,
        pub output_bytes: usize,
        pub input_bytes: usize,
    }

    impl Default for GlkTestWindow {
        fn default() -> Self {
            Self {
                winid: 0,
                width: 12,
                height: 32,
                cursor_x: 0,
                cursor_y: 0,
                textdata: String::new(),
                input_buffer: RefCell::new(Vec::new()),
                input_cursor: RefCell::new(0),
                output_bytes: 0,
                input_bytes: 0,
            }
        }
    }

    impl super::GlkWindow for GlkTestWindow {
        fn new(_request: Receiver<GlkMessage>, _result: Sender<GlkResult>) -> Self {
            Self::default()
        }

        fn run(&mut self) {}

        fn init(&mut self, winid: GlkWindowID) {
            self.winid = winid;
        }

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

        fn get_line(&mut self, _event: LineInput, _initlen: usize, _tx: Sender<GlkEvent>) {
            // no-op
        }

        fn write_char(&mut self, ch: u8) -> usize {
            self.textdata.push(ch as char);
            1
        }

        fn write_string(&mut self, s: &str) -> usize {
            self.textdata.push_str(s);
            s.len()
        }

        fn write_buffer(&mut self, buf: &[u8]) -> usize {
            self.textdata.extend(buf.iter().map(|a| *a as char));
            buf.len()
        }

        fn write_char_uni(&mut self, ch: char) -> usize {
            self.textdata.push(ch);
            4
        }

        fn write_buffer_uni(&mut self, buf: &[char]) -> usize {
            self.textdata.extend(buf.iter());
            4 * buf.len()
        }
    }

    impl GlkTestWindow {
        pub fn set_input_buffer(&mut self, s: &str) {
            self.input_buffer = RefCell::new(Vec::from_iter(s.chars()));
            self.input_cursor = RefCell::new(0);
        }
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;
    use testwin::*;

    #[test]
    fn can_create_window() {
        let mut winsys = WindowManager::<GlkTestWindow>::default();

        let win = winsys.open_window(WindowType::TextBuffer, 32).unwrap();
        let winref = winsys.get_ref(win).unwrap();
        assert_eq!(winref.get_type(), GlkWindowType::TextBuffer);
    }

    #[test]
    fn can_split_window() {
        let mut winsys = WindowManager::<GlkTestWindow>::default();
        let root_win = winsys.open_window(WindowType::TextBuffer, 32).unwrap();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Fixed(3),
            border: false,
        };

        let root_win = winsys.get_ref(root_win).unwrap();
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
        let mut winsys = WindowManager::<GlkTestWindow>::default();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(50),
            border: false,
        };

        let window_a = winsys.open_window(WindowType::TextBuffer, 32).unwrap();
        let window_b = winsys
            .split(window_a, Some(method.clone()), WindowType::TextBuffer, 33)
            .unwrap();
        winsys.split(window_a, Some(method.clone()), WindowType::TextBuffer, 34);

        let wina_ref = winsys.get_ref(window_a).unwrap();
        let sibling = wina_ref.get_sibling().unwrap();
        assert_eq!(sibling.get_rock(), 34);

        let winb_ref = winsys.get_ref(window_b).unwrap();
        let sibling = winb_ref.get_sibling().unwrap();
        assert_eq!(sibling.get_type(), GlkWindowType::Pair);
    }

    #[test]
    fn can_destroy_window() {
        let mut winsys = WindowManager::<GlkTestWindow>::default();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(50),
            border: false,
        };

        let window_a = winsys.open_window(WindowType::TextBuffer, 32).unwrap();
        winsys.split(window_a, Some(method.clone()), WindowType::TextBuffer, 33);

        let window_c = winsys
            .split(window_a, Some(method.clone()), WindowType::TextBuffer, 34)
            .unwrap();
        let window_d = winsys
            .split(window_c, Some(method.clone()), WindowType::TextBuffer, 35)
            .unwrap();

        let wind_ref = winsys.get_ref(window_d).unwrap();
        let parent = wind_ref.get_parent().unwrap();
        let sibling = parent.get_sibling().unwrap();

        assert_eq!(sibling.get_rock(), 32);

        println!("---\nbefore:");
        winsys._dump();
        winsys.close(window_d);
        println!("\n\n---\nafter:");
        winsys._dump();

        assert_eq!(
            winsys
                .get_ref(window_a)
                .unwrap()
                .get_sibling()
                .unwrap()
                .get_rock(),
            34
        );
    }

    #[test]
    fn can_retrieve_window_size() {
        let mut winsys = WindowManager::<GlkTestWindow>::default();
        let root_window = winsys.open_window(WindowType::TextBuffer, 32).unwrap();
        let root_ref = winsys.get_ref(root_window).unwrap();
        let size = root_ref.get_size();
        assert_eq!(size.width, 12);
        assert_eq!(size.height, 32);
    }

    #[test]
    fn can_get_window_constraints() {
        let mut winsys = WindowManager::<GlkTestWindow>::default();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(20),
            border: false,
        };

        let window_a = winsys.open_window(WindowType::TextBuffer, 32).unwrap();
        let window_b = winsys
            .split(window_a, Some(method.clone()), WindowType::TextBuffer, 33)
            .unwrap();

        let winb_ref = winsys.get_ref(window_b).unwrap();
        let (pair_method, _) = winb_ref.get_parent().unwrap().get_arrangement().unwrap();
        assert_eq!(method.position, pair_method.position);
        assert_eq!(method.amount, pair_method.amount);
        assert_eq!(method.border, pair_method.border);
    }

    #[test]
    fn default_to_child2_for_key_window() {
        let mut winsys = WindowManager::<GlkTestWindow>::default();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(20),
            border: false,
        };

        let window_a = winsys.open_window(WindowType::TextBuffer, 32).unwrap();
        let window_b = winsys
            .split(window_a, Some(method.clone()), WindowType::TextBuffer, 33)
            .unwrap();

        let (_, keywin) = winsys
            .get_ref(window_b)
            .unwrap()
            .get_parent()
            .unwrap()
            .get_arrangement()
            .unwrap();
        assert_eq!(keywin.unwrap().id(), window_b);
    }

    #[test]
    fn can_change_key_window_to_child1() {
        let mut winsys = WindowManager::<GlkTestWindow>::default();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(20),
            border: false,
        };

        let window_a = winsys.open_window(WindowType::TextBuffer, 32).unwrap();
        winsys.split(window_a, Some(method.clone()), WindowType::TextBuffer, 33);

        let wina_ref = winsys.get_ref(window_a).unwrap();
        let parent = wina_ref.get_parent().unwrap();
        parent.set_arrangement(method.clone(), Some(&wina_ref));

        let (_, keywin) = parent.get_arrangement().unwrap();
        assert!(Rc::ptr_eq(&keywin.unwrap().winref, &wina_ref.winref));
    }

    #[test]
    fn cannot_get_arrangement_for_non_pair_window() {
        let mut winsys = WindowManager::<GlkTestWindow>::default();
        let window_a = winsys.open_window(WindowType::TextBuffer, 32).unwrap();
        let wina_ref = winsys.get_ref(window_a).unwrap();
        assert!(wina_ref.get_arrangement().is_none());
    }

    #[test]
    fn can_move_cursor_in_a_window() {
        let mut winsys = WindowManager::<GlkTestWindow>::default();
        let window_a = winsys.open_window(WindowType::TextBuffer, 32).unwrap();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(20),
            border: false,
        };
        let window_b = winsys
            .split(window_a, Some(method), WindowType::TextGrid, 55)
            .unwrap();

        // text buffers do not use move_cursor
        let wina_ref = winsys.get_ref(window_a).unwrap();
        wina_ref.move_cursor(4, 4);
        assert_eq!(wina_ref.winref.borrow().window.borrow().cursor_x, 0);

        // text grid windows DO move the cursor
        let winb_ref = winsys.get_ref(window_b).unwrap();
        winb_ref.move_cursor(4, 4);
        assert_eq!(winb_ref.winref.borrow().window.borrow().cursor_x, 4);
    }

    #[test]
    fn can_clear_window() {
        let mut winsys = WindowManager::<GlkTestWindow>::default();
        let window_a = winsys.open_window(WindowType::TextGrid, 32).unwrap();
        let wina_ref = winsys.get_ref(window_a).unwrap();
        wina_ref.move_cursor(5, 5);
        assert_eq!(wina_ref.winref.borrow().window.borrow().cursor_x, 5);
        wina_ref.clear();
        assert_eq!(wina_ref.winref.borrow().window.borrow().cursor_x, 0);
    }

    #[test]
    fn can_set_input_buffer_in_test_window() {
        let mut winsys = WindowManager::<GlkTestWindow>::default();
        let window_a = winsys.open_window(WindowType::TextGrid, 32).unwrap();
        let wina_ref = winsys.get_ref(window_a).unwrap();
        wina_ref
            .winref
            .borrow()
            .window
            .borrow_mut()
            .set_input_buffer("test buffer");
        assert_eq!(
            wina_ref
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
}
*/
