use std::{cell::RefCell, rc::Rc};

/// A thread-local, cloneable event queue backed by `Rc<RefCell<Vec<WindowEvent>>>`.
///
/// Cloning shares the underlying storage — events pushed through one handle
/// are visible to all clones.
#[derive(Debug, Clone)]
pub struct WindowEventQueue {
    inner: Rc<RefCell<Vec<WindowEvent>>>,
}

impl WindowEventQueue {
    /// Creates an empty event queue.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Vec::<WindowEvent>::new())),
        }
    }

    /// Pushes a single event into the queue.
    pub fn push(&self, event: WindowEvent) {
        self.inner.borrow_mut().push(event);
    }

    /// Drains all pending events and returns them as a `Vec`.
    pub fn drain(&self) -> Vec<WindowEvent> {
        self.inner.borrow_mut().drain(..).collect()
    }

    /// Returns `true` if the queue contains no events.
    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }
}

/// Events that can be produced by a [`LayerShellWindow`](crate::window::LayerShellWindow).
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// A keyboard key was pressed or released.
    Keyboard(KeyEvent),
    /// A pointer (mouse) event occurred.
    Pointer(MouseEvent),
    /// The window was resized by the compositor.
    Resize { width: u32, height: u32 },
}

/// A keyboard key press or release event.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KeyEvent {
    /// The logical key code.
    pub code: KeyCode,
    /// `1` for press, `0` for release.
    pub value: u32,
    /// Whether Shift was held.
    pub shift: bool,
    /// Whether Alt was held.
    pub alt: bool,
    /// Whether Ctrl was held.
    pub ctrl: bool,
}

/// A pointer (mouse) event.
#[derive(Debug, Clone, PartialEq)]
pub enum MouseEvent {
    /// The pointer moved to a cell position.
    Motion { row: u32, col: u32 },
    /// The pointer entered the window's surface.
    Enter { row: u32, col: u32 },
    /// The pointer left the window's surface.
    Leave,
    /// A mouse button was pressed or released.
    Button { code: ButtonCode, value: u32 },
    /// A scroll axis was moved.
    Axis { code: AxisCode, value: f64 },
}

/// Logical key codes representing physical keys on a keyboard.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum KeyCode {
    /// A printable character.
    Char(char),
    /// A function key (F1–F12).
    F(u8),
    /// The Backspace key.
    Backspace,
    /// The Enter / Return key.
    Enter,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// The Tab key.
    Tab,
    /// The Delete key.
    Delete,
    /// The Home key.
    Home,
    /// The End key.
    End,
    /// The Page Up key.
    PageUp,
    /// The Page Down key.
    PageDown,
    /// The Escape key.
    Esc,
    /// A key that could not be translated to a known code.
    Unidentified,
}

/// Mouse button codes.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ButtonCode {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button.
    Middle,
    /// An unrecognised button.
    Unknown,
}

/// Scroll axis identifiers.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AxisCode {
    /// Vertical scrolling (wheel up/down).
    VerticalScroll,
    /// Horizontal scrolling (wheel left/right).
    HorizontalScroll,
}
