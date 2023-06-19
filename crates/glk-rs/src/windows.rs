use crate::GlkRock;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct WindowRef {
    winref: Rc<RefCell<Window>>,
}

impl WindowRef {
    /// Set up windows subsystem
    pub fn init() -> WindowRef {
        WindowRef {
            winref: Rc::new(RefCell::new(Window {
                wintype: WindowType::Root,
                rock: 0,
                parent: None,
                child1: None,
                child2: None,
            })),
        }
    }

    /// Create a new window
    pub fn create_root(&self, wintype: WindowType, rock: GlkRock) -> WindowRef {
        assert!(self.winref.borrow().wintype == WindowType::Root);
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
            .replace(Rc::downgrade(&self.winref));
        self.winref
            .borrow_mut()
            .child1
            .replace(root_win.make_clone());
        root_win
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
}

/// A glk window
#[derive(Debug)]
pub struct Window {
    wintype: WindowType,
    rock: GlkRock,
    parent: Option<Weak<RefCell<Window>>>,
    child1: Option<WindowRef>,
    child2: Option<WindowRef>,
}

pub struct WindowSplitMethod {
    /// Location of new window in relation to the existing window
    position: WindowSplitPosition,

    /// What the new window should look like compared to the existing one
    amount: WindowSplitAmount,

    /// Does it have a border?
    border: bool,
}

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

pub enum WindowSplitAmount {
    /// New window should have a fixed number of lines/columns
    Fixed(i32),

    /// New window should consume a percentage of the existing window
    Proportional(i32),
}

#[derive(Debug, PartialEq, Copy, Clone)]
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
    Root,
}

impl Window {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_create_window() {
        let winsys = WindowRef::init();
        let win = winsys.create_root(WindowType::TextBuffer, 32);
        assert_eq!(win.get_type(), WindowType::TextBuffer);
    }

    #[test]
    fn can_split_window() {
        let winsys = WindowRef::init();
        let root_win = winsys.create_root(WindowType::TextBuffer, 32);

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Fixed(3),
            border: false,
        };
        let split = root_win.split(Some(method), WindowType::TextBuffer, 65);
        let parent = split.get_parent();
        assert_eq!(split.get_parent().unwrap().get_rock(), 0);
    }
}
