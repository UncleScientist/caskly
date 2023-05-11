use std::{cell::RefCell, rc::Rc};

/// An opaque window ID type
pub struct WinID {
    id: Rc<RefCell<Window>>,
}

/// A window in glk
pub(crate) struct Window {
    /// the parent of this window
    parent: Option<WinID>,

    /// first child of this window
    child1: Option<WinID>,

    /// second child of this window
    child2: Option<WinID>,

    /// direction to split window
    split_dir: SplitDirection,

    /// method to determine size of new window
    split_amt: SplitMethod,

    /// style hint for the border
    border: BorderStyle,

    /// type of window being created
    wintype: WindowType,

    /// The rock defined by the application
    rock: u32,
}

impl WinID {
    pub(crate) fn new(
        split_dir: SplitDirection,
        split_amt: SplitMethod,
        border: BorderStyle,
        wintype: WindowType,
        rock: u32,
    ) -> Self {
        let win = Window {
            parent: None,
            child1: None,
            child2: None,
            split_dir,
            split_amt,
            border,
            wintype,
            rock,
        };

        Self {
            id: Rc::new(RefCell::new(win)),
        }
    }

    pub(crate) fn get_clone(&self) -> Self {
        Self {
            id: Rc::clone(&self.id),
        }
    }

    /// get the rock value for this window
    pub(crate) fn get_rock(&self) -> u32 {
        self.id.borrow().rock
    }

    pub(crate) fn parent(&self) -> Option<Self> {
        Some(Self {
            id: Rc::clone(&self.id.borrow().parent.as_ref()?.id),
        })
    }

    pub(crate) fn sibling(&self) -> Option<WinID> {
        let parent = Rc::clone(&self.id.borrow().parent.as_ref()?.id);
        let c1 = Rc::clone(&parent.borrow().child1.as_ref()?.id);
        let c2 = Rc::clone(&parent.borrow().child2.as_ref()?.id);

        if Rc::ptr_eq(&c1, &self.id) {
            Some(Self { id: c1 })
        } else {
            Some(Self { id: c2 })
        }
    }
}

/// Types of windows
#[derive(Debug, PartialEq)]
pub enum WindowType {
    /// a text buffer window -- can stream output, and accept line input
    TextBuffer,

    /// A text grid window -- can draw characters at arbitrary x/y coordinates
    TextGrid,

    /// Can display graphics
    Graphics,

    /// A basic window with no input or output facility
    Blank,

    /// A "pair" window (used only internally by glk)
    Pair,
}

/// Direction to split a window
#[derive(Debug, PartialEq)]
pub enum SplitDirection {
    /// the new window appears Above the old window
    Above,
    /// the new window appears Below the old window
    Below,
    /// the new window appears to the Left the old window
    Left,
    /// the new window appears to the Right the old window
    Right,
}

/// The amount of space that is given to the new window
#[derive(Debug, PartialEq)]
pub enum SplitMethod {
    /// create a window with a width or height value given in characters
    Fixed(u32),

    /// create a window with a width or hight as a percentage of the original window
    Proportional(u32),
}

/// Whether or not to put a border around the window
pub type BorderStyle = bool;
