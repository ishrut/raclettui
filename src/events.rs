use xkbcommon::xkb;
use std::{cell::RefCell, rc::Rc};

/// A thread-local, reference-counted event queue for window events.
///
/// Internally uses `Rc<RefCell<Vec<T>>>` to allow multiple clones of the queue
/// to push and drain events safely within the same thread.
#[derive(Debug, Clone)]
pub struct WindowEventQueue {
    /// Internal, shared queue wrapped in `Rc<RefCell<...>>`.
    inner: Rc<RefCell<Vec<WindowEvent>>>,
}

impl WindowEventQueue {
    /// Creates a new empty event queue.
    ///
    /// # Example
    /// ```
    /// let queue: WindowEventQueue<WindowEvent> = WindowEventQueue::new();
    /// ```
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Vec::<WindowEvent>::new())),
        }
    }

    /// Clone the queue, producing a new handle pointing to the same underlying vector.
    ///
    /// This allows pushing events from multiple places while keeping a single
    /// logical queue.
    pub fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone()
        }
    }

    /// Add an event to the queue.
    ///
    /// # Parameters
    /// * `event` - The event to push.
    pub fn push(&self, event: WindowEvent) {
        self.inner.borrow_mut().push(event);
    }

    /// Remove and return all events currently in the queue.
    ///
    /// # Returns
    /// A `Vec<WindowEvent>` containing all drained events.
    pub fn drain(&self) -> Vec<WindowEvent> {
        self.inner.borrow_mut().drain(..).collect()
    }

    /// Check whether the queue is empty.
    ///
    /// # Returns
    /// `true` if there are no events in the queue, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }

}

/// Events that can occur on a window.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// A keyboard-related event.
    Keyboard(KeyEvent),

    /// A pointer-related event.
    Pointer(MouseEvent),

    /// The window has been resized.
    Resize {
        width: u32,
        height: u32
    },
}

impl WindowEvent {
    /// Create a new keyboard event from an `xkbcommon::xkb::State` and keycode/value.
    ///
    /// # Parameters
    /// * `keymap_state` - The current XKB state.
    /// * `key` - The hardware keycode (usually from Wayland).
    /// * `value` - Key press/release state (often 0=release, 1=press).
    ///
    /// # Returns
    /// A `WindowEvent::Keyboard` containing a populated `KeyEvent`.
    pub fn new_keyboard_event(keymap_state: &xkb::State, key: u32, value: u32) -> WindowEvent {
        let keysym = keymap_state.key_get_one_sym(xkb::Keycode::new(key+8));

        let shift = keymap_state.mod_name_is_active(xkb::MOD_NAME_SHIFT, xkb::STATE_MODS_EFFECTIVE);
        let ctrl = keymap_state.mod_name_is_active(xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE);
        let alt = keymap_state.mod_name_is_active(xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE);

        let keycode = match keysym {
            xkb::Keysym::BackSpace => KeyCode::Backspace,
            xkb::Keysym::Return => KeyCode::Enter,
            xkb::Keysym::Left => KeyCode::Left,
            xkb::Keysym::Right => KeyCode::Right,
            xkb::Keysym::Up => KeyCode::Up,
            xkb::Keysym::Down => KeyCode::Down,
            xkb::Keysym::Tab => KeyCode::Tab,
            xkb::Keysym::Delete => KeyCode::Delete,
            xkb::Keysym::Home => KeyCode::Home,
            xkb::Keysym::End => KeyCode::End,
            xkb::Keysym::Page_Up => KeyCode::PageUp,
            xkb::Keysym::Page_Down => KeyCode::PageDown,
            xkb::Keysym::Escape => KeyCode::Esc,
            xkb::Keysym::F1 => KeyCode::F(1),
            xkb::Keysym::F2 => KeyCode::F(2),
            xkb::Keysym::F3 => KeyCode::F(3),
            xkb::Keysym::F4 => KeyCode::F(4),
            xkb::Keysym::F5 => KeyCode::F(5),
            xkb::Keysym::F6 => KeyCode::F(6),
            xkb::Keysym::F7 => KeyCode::F(7),
            xkb::Keysym::F8 => KeyCode::F(8),
            xkb::Keysym::F9 => KeyCode::F(9),
            xkb::Keysym::F10 => KeyCode::F(10),
            xkb::Keysym::F11 => KeyCode::F(11),
            xkb::Keysym::F12 => KeyCode::F(12),
            _ =>  {
                let utf32 = xkb::keysym_to_utf32(keysym);
                if utf32 != 0 {
                    let ch = std::char::from_u32(utf32).unwrap();
                    KeyCode::Char(ch)
                } else {
                    KeyCode::Unidentified
                }
            }
        };
        WindowEvent::Keyboard(KeyEvent {
            code: keycode,
            value,
            shift,
            alt,
            ctrl
        })
    }

    /// Create a new pointer motion event.
    pub fn new_pointer_motion_event(x: f64, y: f64) -> Self {
        Self::Pointer(MouseEvent::Motion { x, y })
    }

    /// Create a pointer enter event.
    pub fn new_pointer_enter_event(x: f64, y: f64) -> Self {
        Self::Pointer(MouseEvent::Enter { x, y })
    }

    /// Create a pointer leave event.
    pub fn new_pointer_leave_event() -> Self {
        Self::Pointer(MouseEvent::Leave)
    }

    /// Create a pointer axis (scroll) event.
    pub fn new_pointer_axis_event(code: AxisCode, value: f64) -> Self {
        Self::Pointer(MouseEvent::Axis { code, value })
    }

    /// Create a pointer button press/release event.
    ///
    /// # Parameters
    /// * `code` - The button code from the hardware.
    /// * `value` - Press/release state.
    /// Maps Linux evdev button codes to semantic ButtonCode.
    /// Note: Wayland sends only u32 button codes; these numbers are common defaults.
    pub fn new_pointer_button_event(code: u32, value: u32) -> Self {
        let button_code = match code {
            272 => ButtonCode::Left,
            273 => ButtonCode::Right,
            274 => ButtonCode::Middle,
            _ => ButtonCode::Unknown,
        };
        Self::Pointer(MouseEvent::Button { code: button_code, value })
    }

    /// Create a window resize event.
    pub fn new_resize_event(width: u32, height: u32) -> Self {
        Self::Resize { width, height }
    }
}

/// Keyboard event details.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyEvent {
    /// Logical key code (maps physical key to semantic meaning).
    pub code: KeyCode,

    /// Key press/release value (often 0=release, 1=press).
    pub value: u32,

    /// Whether the shift modifier is active.
    pub shift: bool,

    /// Whether the alt modifier is active.
    pub alt: bool,

    /// Whether the ctrl modifier is active.
    pub ctrl: bool,
}

/// Pointer (mouse) events.
#[derive(Debug, Clone, PartialEq)]
pub enum MouseEvent {
    /// Mouse moved to a specific position.
    Motion { x: f64, y: f64 },

    /// Pointer entered the window at a position.
    Enter { x: f64, y: f64 },

    /// Pointer left the window.
    Leave,

    /// Mouse button pressed/released.
    Button { code: ButtonCode, value: u32 },

    /// Axis event (scrolling).
    Axis { code: AxisCode, value: f64 },
}

/// Logical key codes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum KeyCode {
    /// Printable character.
    Char(char),

    /// Function keys F1-F12.
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

    /// Unknown or unmapped key.
    Unidentified,
}

/// Mouse button codes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ButtonCode {
    Left,
    Right,
    Middle,
    Unknown,
}

/// Axis (scroll) types for pointer events.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AxisCode {
    VerticalScroll,
    HorizontalScroll,
}
