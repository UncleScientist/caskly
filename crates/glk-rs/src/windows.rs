use crate::GlkRock;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug, Default)]
pub struct StreamResult {
    readcount: u32,
    writecount: u32,
}

#[derive(Debug, Default)]
pub struct WindowRef {
    winref: Rc<RefCell<Window>>,
}

#[derive(Debug, Default)]
pub(crate) struct WindowManager {
    root: WindowRef,
}

impl WindowManager {
    /// Create a new window
    pub(crate) fn open_window(&self, wintype: WindowType, rock: GlkRock) -> WindowRef {
        assert!(self.root.winref.borrow().wintype == WindowType::Root);
        let root_win = WindowRef {
            winref: Rc::new(RefCell::new(Window {
                wintype,
                rock,
                parent: None,
                child1: None,
                child2: None,
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

    /// Close an existing window
    pub(crate) fn close_window(&self, win: &WindowRef) -> StreamResult {
        if let Some(child) = &win.winref.borrow().child1 {
            self.close_window(child);
        }

        if let Some(child) = &win.winref.borrow().child2 {
            self.close_window(child);
        }

        win.winref.borrow_mut().child1 = None;
        win.winref.borrow_mut().child2 = None;

        StreamResult::default()
    }

    fn dump(&self) {
        self.root.dump(4);
    }
}

impl WindowRef {
    fn dump(&self, indent: usize) {
        println!(
            "{:indent$}{:?} ({})",
            "",
            self.winref.borrow().wintype,
            self.winref.borrow().rock
        );
        if let Some(child) = &self.winref.borrow().child1 {
            child.dump(indent + 4);
        }

        if let Some(child) = &self.winref.borrow().child2 {
            child.dump(indent + 4);
        }
    }

    // before:                after:
    //     W                     P
    //                          / \
    //                         W   N
    /// Split an existing window. Creates a pair window which becomes the
    /// parent of the two windows. The original parent of the split window
    /// becomes the parent of the pair window, and the window being split
    /// becomes the sibling of the new window being created.
    pub fn split(
        &self,
        method: Option<WindowSplitMethod>,
        wintype: WindowType,
        rock: GlkRock,
    ) -> WindowRef {
        if self.winref.borrow().wintype == WindowType::Root {
            let child = WindowRef {
                winref: Rc::new(RefCell::new(Window {
                    wintype,
                    rock,
                    parent: None,
                    child1: None,
                    child2: None,
                })),
            };
            self.winref.borrow_mut().child1.replace(child.make_clone());
            return child;
        }

        let new_win = WindowRef {
            winref: Rc::new(RefCell::new(Window {
                wintype,
                rock,
                parent: None,
                child1: None,
                child2: None,
            })),
        };
        let pair_win = WindowRef {
            winref: Rc::new(RefCell::new(Window {
                wintype: WindowType::Pair,
                rock: 0,
                parent: None,
                child1: Some(self.make_clone()),
                child2: Some(new_win.make_clone()),
            })),
        };
        if let Some(old_parent) = self
            .winref
            .borrow_mut()
            .parent
            .replace(Rc::downgrade(&pair_win.winref))
        {
            pair_win
                .winref
                .borrow_mut()
                .parent
                .replace(old_parent.clone());
            old_parent
                .upgrade()
                .unwrap()
                .borrow_mut()
                .child1
                .replace(pair_win.make_clone());
        } else {
            panic!("missing root window");
        }
        new_win
            .winref
            .borrow_mut()
            .parent
            .replace(Rc::downgrade(&pair_win.winref));
        new_win
    }

    fn make_clone(&self) -> WindowRef {
        WindowRef {
            winref: Rc::clone(&self.winref),
        }
    }

    fn get_type(&self) -> WindowType {
        self.winref.borrow().wintype
    }

    fn get_rock(&self) -> GlkRock {
        self.winref.borrow().rock
    }

    fn get_parent(&self) -> Option<WindowRef> {
        Some(WindowRef {
            winref: self.winref.borrow().parent.as_ref()?.upgrade()?,
        })
    }

    fn get_sibling(&self) -> Option<WindowRef> {
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
}

/// A glk window
#[derive(Debug, Default)]
pub struct Window {
    wintype: WindowType,
    rock: GlkRock,
    parent: Option<Weak<RefCell<Window>>>,
    child1: Option<WindowRef>,
    child2: Option<WindowRef>,
}

#[derive(Copy, Clone)]
pub struct WindowSplitMethod {
    /// Location of new window in relation to the existing window
    position: WindowSplitPosition,

    /// What the new window should look like compared to the existing one
    amount: WindowSplitAmount,

    /// Does it have a border?
    border: bool,
}

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
pub enum WindowSplitAmount {
    /// New window should have a fixed number of lines/columns
    Fixed(i32),

    /// New window should consume a percentage of the existing window
    Proportional(i32),
}

#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub enum WindowType {
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

impl Window {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_create_window() {
        let winsys = WindowManager::default();

        let win = winsys.open_window(WindowType::TextBuffer, 32);
        assert_eq!(win.get_type(), WindowType::TextBuffer);
    }

    #[test]
    fn can_split_window() {
        let winsys = WindowManager::default();
        let root_win = winsys.open_window(WindowType::TextBuffer, 32);

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Fixed(3),
            border: false,
        };

        let split = root_win.split(Some(method), WindowType::TextBuffer, 65);

        let parent = split.get_parent().unwrap();
        assert_eq!(parent.get_rock(), 0);

        let sibling = split.get_sibling().unwrap();
        assert_eq!(sibling.get_rock(), 32);

        let sibling = root_win.get_sibling().unwrap();
        assert_eq!(sibling.get_rock(), 65);
    }

    #[test]
    fn can_split_multiple_times() {
        let winsys = WindowManager::default();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(50),
            border: false,
        };

        let window_a = winsys.open_window(WindowType::TextBuffer, 32);
        let window_b = window_a.split(Some(method.clone()), WindowType::TextBuffer, 33);
        let _window_c = window_a.split(Some(method.clone()), WindowType::TextBuffer, 34);

        let sibling = window_a.get_sibling().unwrap();
        assert_eq!(sibling.get_rock(), 34);

        let sibling = window_b.get_sibling().unwrap();
        assert_eq!(sibling.get_type(), WindowType::Pair);
    }

    #[test]
    fn can_destroy_window() {
        let winsys = WindowManager::default();

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Proportional(50),
            border: false,
        };

        let window_a = winsys.open_window(WindowType::TextBuffer, 32);
        let window_b = window_a.split(Some(method.clone()), WindowType::TextBuffer, 33);

        let window_c = window_a.split(Some(method.clone()), WindowType::TextBuffer, 34);
        let window_d = window_c.split(Some(method.clone()), WindowType::TextBuffer, 35);

        let parent = window_d.get_parent().unwrap();
        let sibling = parent.get_sibling().unwrap();
        winsys.dump();
        assert_eq!(sibling.get_rock(), 32);

        // TODO: call window_d.destroy();
    }
}
