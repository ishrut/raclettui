//! Keyboard, pointer and window events.
//!
//! Events are collected while the window waits for the compositor (inside
//! ratatui's `Terminal::draw`, or [`CpuWindow::update`](crate::CpuWindow::update) /
//! [`WgpuWindow::update`](crate::WgpuWindow::update)) and pushed to a
//! [`WindowEventQueue`]. Get a handle to the queue with `get_event_queue()`
//! before handing the window to ratatui, then drain it after every frame:
//!
//! ```no_run
//! use raclettui::{WindowBuilder, Anchor, KeyboardInteractivity};
//! use raclettui::events::{KeyCode, WindowEvent};
//! use ratatui::{Terminal, widgets::Paragraph};
//!
//! let window = WindowBuilder::new()
//!     .set_width(400)
//!     .set_height(200)
//!     .set_anchor(Anchor::Top)
//!     .set_keyboard_interactivity(KeyboardInteractivity::OnDemand)
//!     .init_cpu()?;
//! let events = window.get_event_queue();
//! let mut terminal = Terminal::new(window)?;
//!
//! 'app: loop {
//!     terminal.draw(|f| f.render_widget(Paragraph::new("press q"), f.area()))?;
//!     for event in events.drain() {
//!         match event {
//!             WindowEvent::Keyboard(key) if key.is_press() && key.code == KeyCode::Char('q') => break 'app,
//!             WindowEvent::Closed => break 'app,
//!             _ => {}
//!         }
//!     }
//! }
//! # Ok::<(), raclettui::Error>(())
//! ```

use xkbcommon::xkb;
use std::{cell::RefCell, rc::Rc};

/// A single-threaded, reference-counted event queue for window events.
///
/// Internally uses `Rc<RefCell<Vec<WindowEvent>>>`; cloning the queue
/// produces a new handle to the same underlying events, so the window can
/// push events while the application drains them.
#[derive(Debug, Clone, Default)]
pub struct WindowEventQueue {
    /// Internal, shared queue wrapped in `Rc<RefCell<...>>`.
    inner: Rc<RefCell<Vec<WindowEvent>>>,
}

impl WindowEventQueue {
    /// Creates a new empty event queue.
    ///
    /// # Example
    /// ```
    /// use raclettui::events::{WindowEvent, WindowEventQueue};
    ///
    /// let queue = WindowEventQueue::new();
    /// let handle = queue.clone();
    /// handle.push(WindowEvent::new_resize_event(800, 600));
    /// assert_eq!(queue.drain(), vec![WindowEvent::Resize { width: 800, height: 600 }]);
    /// assert!(queue.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an event to the queue.
    ///
    /// # Parameters
    /// * `event` - The event to push.
    pub fn push(&self, event: WindowEvent) {
        self.inner.borrow_mut().push(event);
    }

    /// Remove and return all events currently in the queue, oldest first.
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

    /// The compositor resized the window. `width` and `height` are in pixels.
    ///
    /// The ratatui grid is resized automatically on the next draw.
    Resize {
        /// New width in pixels.
        width: u32,
        /// New height in pixels.
        height: u32
    },

    /// The compositor closed the window, e.g. because its output was
    /// disconnected. Drawing to the window returns
    /// [`Error::WindowClosed`](crate::Error::WindowClosed) afterwards, so
    /// the application should exit or create a new window.
    Closed,
}

impl WindowEvent {
    /// Create a new keyboard event from an `xkbcommon::xkb::State` and keycode/value.
    ///
    /// # Parameters
    /// * `keymap_state` - The current XKB state.
    /// * `key` - The hardware keycode (usually from Wayland).
    /// * `value` - Key state: 0 = release, 1 = press, 2 = repeat.
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
                match std::char::from_u32(xkb::keysym_to_utf32(keysym)) {
                    Some(ch) if ch != '\0' => KeyCode::Char(ch),
                    _ => KeyCode::Unidentified,
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
    /// Maps Linux evdev button codes (`BTN_LEFT` = 272, `BTN_RIGHT` = 273,
    /// `BTN_MIDDLE` = 274) to a [`ButtonCode`].
    ///
    /// # Parameters
    /// * `code` - The evdev button code sent by the compositor.
    /// * `value` - Button state: 0 = released, 1 = pressed.
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
///
/// Both presses and releases are reported, check [`is_press`](Self::is_press)
/// to react to a key only once.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyEvent {
    /// Logical key code (maps physical key to semantic meaning).
    pub code: KeyCode,

    /// Key state: 0 = release, 1 = press, 2 = repeat (sent by compositors
    /// implementing `wl_seat` version 10).
    pub value: u32,

    /// Whether the shift modifier is active.
    pub shift: bool,

    /// Whether the alt modifier is active.
    pub alt: bool,

    /// Whether the ctrl modifier is active.
    pub ctrl: bool,
}

impl KeyEvent {
    /// Returns `true` if the key was pressed.
    pub fn is_press(&self) -> bool {
        self.value == 1
    }

    /// Returns `true` if the key was released.
    pub fn is_release(&self) -> bool {
        self.value == 0
    }

    /// Returns `true` if this is a compositor generated key repeat.
    pub fn is_repeat(&self) -> bool {
        self.value == 2
    }
}

/// Pointer (mouse) events.
///
/// Coordinates are surface-local, in logical pixels.
#[derive(Debug, Clone, PartialEq)]
pub enum MouseEvent {
    /// Mouse moved to a specific position.
    Motion {
        /// Horizontal position.
        x: f64,
        /// Vertical position.
        y: f64,
    },

    /// Pointer entered the window at a position.
    Enter {
        /// Horizontal position.
        x: f64,
        /// Vertical position.
        y: f64,
    },

    /// Pointer left the window.
    Leave,

    /// Mouse button pressed (`value` = 1) or released (`value` = 0).
    Button {
        /// Which button changed.
        code: ButtonCode,
        /// 1 = pressed, 0 = released.
        value: u32,
    },

    /// Axis event (scrolling). Positive values scroll down / right.
    Axis {
        /// Scroll direction.
        code: AxisCode,
        /// Scroll distance in logical pixels.
        value: f64,
    },
}

/// Logical key codes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum KeyCode {
    /// Printable character.
    Char(char),

    /// Function keys F1-F12.
    F(u8),

    /// Backspace key.
    Backspace,
    /// Enter / Return key.
    Enter,
    /// Left arrow key.
    Left,
    /// Right arrow key.
    Right,
    /// Up arrow key.
    Up,
    /// Down arrow key.
    Down,
    /// Tab key.
    Tab,
    /// Delete key.
    Delete,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page up key.
    PageUp,
    /// Page down key.
    PageDown,
    /// Escape key.
    Esc,

    /// Unknown or unmapped key.
    Unidentified,
}

/// Mouse button codes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ButtonCode {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button / wheel click.
    Middle,
    /// Any other button.
    Unknown,
}

/// Axis (scroll) types for pointer events.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AxisCode {
    /// Vertical scroll wheel / gesture.
    VerticalScroll,
    /// Horizontal scroll wheel / gesture.
    HorizontalScroll,
}
