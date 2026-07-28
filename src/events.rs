use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct WindowEventQueue {
    inner: Rc<RefCell<Vec<WindowEvent>>>,
}

impl WindowEventQueue {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Vec::<WindowEvent>::new())),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone()
        }
    }

    pub fn push(&self, event: WindowEvent) {
        self.inner.borrow_mut().push(event);
    }

    pub fn drain(&self) -> Vec<WindowEvent> {
        self.inner.borrow_mut().drain(..).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }

}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    Keyboard(KeyEvent),
    Pointer(MouseEvent),
    Resize {
        width: u32,
        height: u32
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub value: u32,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseEvent {
    Motion { row: u32, col: u32 },
    Enter { row: u32, col: u32 },
    Leave,
    Button { code: ButtonCode, value: u32 },
    Axis { code: AxisCode, value: f64 },
}

/// Logical key codes.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum KeyCode {
    Char(char),
    F(u8),
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Tab,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Esc,
    Unidentified,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ButtonCode {
    Left,
    Right,
    Middle,
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AxisCode {
    VerticalScroll,
    HorizontalScroll,
}
