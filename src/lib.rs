//! # raclettui
//!
//! A [ratatui] backend that renders TUI widgets on a **Wayland layer-shell**
//! window using GPU-accelerated glyph rendering via [beamterm-core].
//!
//! Instead of running in a terminal emulator, raclettui creates its own
//! window (overlay, panel, background, etc.) and draws directly with
//! OpenGL. This is useful for status bars, HUDs, lock screens, and other
//! pop-up / overlay UIs on Wayland compositors that support the
//! [`zwlr_layer_shell_v1`] protocol (e.g. Sway, Hyprland, River).
//!
//! ## Quick start
//!
//! ```no_run
//! use raclettui::{
//!     WindowBuilder, KeyCode, WindowEvent,
//!     layer_shell::{Anchor, KeyboardInteractivity, Layer},
//! };
//! use ratatui::{Terminal, widgets::Paragraph};
//!
//! // 1. Build and open a layer-shell window.
//! let window = WindowBuilder::default()
//!     .set_keyboard_interactivity(KeyboardInteractivity::OnDemand)
//!     .set_anchors(Anchor::Top | Anchor::Left | Anchor::Right)
//!     .set_height(30)
//!     .build()
//!     .expect("failed to create window");
//!
//! // 2. Clone the event queue for polling.
//! let events = window.events();
//!
//! // 3. Wrap in a ratatui Terminal.
//! let mut terminal = Terminal::new(window).unwrap();
//!
//! // 4. Main loop — draw and handle events.
//! loop {
//!     terminal.draw(|f| {
//!         let paragraph = Paragraph::new("Hello from raclettui!");
//!         f.render_widget(paragraph, f.area());
//!     }).unwrap();
//!
//!     for event in events.drain() {
//!         if let WindowEvent::Keyboard(key) = event {
//!             if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
//!                 return;
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Architecture
//!
//! 1. [`WindowBuilder`] connects to the Wayland display, binds the
//!    `zwlr_layer_shell_v1` global, and creates a layer surface.
//! 2. An EGL/OpenGL context is created on that surface via `glutin`.
//! 3. [beamterm-core] rasterises glyphs into a dynamic font atlas and
//!    renders the terminal grid as textured quads.
//! 4. [`LayerShellWindow`] implements [`ratatui_core::backend::Backend`],
//!    so it can be wrapped in a `ratatui::Terminal`.
//! 5. Wayland input events (keyboard, pointer) are translated into
//!    [`WindowEvent`]s and pushed into a shared [`WindowEventQueue`].
//!
//! ## Event handling
//!
//! Call [`LayerShellWindow::events()`] to get a cloneable
//! [`WindowEventQueue`]. Drain it each frame to process input:
//!
//! ```no_run
//! # use raclettui::WindowEventQueue;
//! # let events = WindowEventQueue::new();
//! for event in events.drain() {
//!     match event {
//!         WindowEvent::Keyboard(k) => { /* k.code, k.shift, k.ctrl, … */ }
//!         WindowEvent::Pointer(m)  => { /* motion, clicks, scroll */ }
//!         WindowEvent::Resize { width, height } => {
//!             // window.resize_grid(width, height);
//!         }
//!     }
//! }
//! ```
//!
//! ## Cargo features
//!
//! This crate currently has no optional features. All dependencies are
//! required.
//!
//! [ratatui]: https://crates.io/crates/ratatui
//! [beamterm-core]: https://crates.io/crates/beamterm-core
//! [`zwlr_layer_shell_v1`]: https://wayland.app/protocols/wlr-layer-shell-unstable-v1

/// Errors produced by the library.
pub mod error;
/// Wayland window events (keyboard, mouse, resize).
pub mod events;
mod ratatui_backend;
/// Wayland layer-shell window creation and management.
pub mod window;

/// Re-exports of the Wayland layer-shell protocol types.
pub mod layer_shell {
    pub use wayland_protocols_wlr::layer_shell::v1::client::{
        zwlr_layer_shell_v1::Layer,
        zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity},
    };
}

pub use events::*;
pub use layer_shell::{Anchor, KeyboardInteractivity, Layer};
pub use window::{LayerShellWindow, WindowBuilder};
