use crate::GlkRock;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct WindowRef {
    winref: Rc<RefCell<Window>>,
}

impl WindowRef {
    /// Create a new window
    pub fn open(
        split: &Option<WindowRef>,
        method: Option<WindowSplitMethod>,
        wintype: WindowType,
        rock: GlkRock,
    ) -> WindowRef {
        WindowRef {
            winref: Rc::new(RefCell::new(Window {
                wintype,
                rock,
                parent: None,
                child1: None,
                child2: None,
            })),
        }
    }

    pub fn split(
        &self,
        method: Option<WindowSplitMethod>,
        wintype: WindowType,
        rock: GlkRock,
    ) -> WindowRef {
        WindowRef {
            winref: Rc::new(RefCell::new(Window {
                wintype,
                rock,
                parent: Some(Rc::downgrade(&self.winref)),
                child1: None,
                child2: None,
            })),
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
}

impl Window {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_create_window() {
        let win = WindowRef::open(&None, None, WindowType::TextBuffer, 0);
        assert_eq!(win.get_type(), WindowType::TextBuffer);
    }

    #[test]
    fn can_split_window() {
        let root_win = WindowRef::open(&None, None, WindowType::TextBuffer, 32);

        let method = WindowSplitMethod {
            position: WindowSplitPosition::Above,
            amount: WindowSplitAmount::Fixed(3),
            border: false,
        };
        let split = root_win.split(Some(method), WindowType::TextBuffer, 65);
        assert_eq!(split.get_parent().unwrap().get_rock(), root_win.get_rock());
    }
}
