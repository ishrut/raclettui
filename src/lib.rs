//! # raclettui
//!
//! Build terminal-themed [Wayland layer shell] applications with [ratatui].
//!
//! raclettui creates a `wlr-layer-shell` surface and implements ratatui's
//! [`Backend`](ratatui_core::backend::Backend) trait for it, so any ratatui
//! application can be drawn into a bar, panel, launcher, notification or
//! overlay by swapping its backend.
//!
//! The crate is still under development: expect bugs and breaking changes.
//!
//! ## Backends
//!
//! Two rendering backends are available, both created from a [`WindowBuilder`]:
//!
//! | Backend | Created with | Rendering |
//! |---------|--------------|-----------|
//! | [`CpuWindow`] | [`WindowBuilder::init_cpu`] | Software rasterization (`fontdue`) into a shared memory buffer. No GPU needed. |
//! | [`WgpuWindow`] | [`WindowBuilder::init_wgpu`] | Hardware accelerated with `wgpu` and `glyphon`, supports bold and italic faces. |
//!
//! ## Example
//!
//! ```no_run
//! use raclettui::{WindowBuilder, Anchor, Layer, KeyboardInteractivity};
//! use raclettui::events::{KeyCode, WindowEvent};
//! use ratatui::{Terminal, widgets::{Block, Paragraph}};
//!
//! fn main() -> Result<(), raclettui::Error> {
//!     let window = WindowBuilder::new()
//!         .set_namespace("hello")
//!         .set_width(400)
//!         .set_height(200)
//!         .set_layer(Layer::Top)
//!         .set_anchor(Anchor::Top)
//!         .set_keyboard_interactivity(KeyboardInteractivity::OnDemand)
//!         .set_bg_alpha(0.8)
//!         .init_cpu()?; // or .init_wgpu()? for gpu rendering
//!
//!     // grab the event queue before the terminal takes ownership of the window
//!     let events = window.get_event_queue();
//!     let mut terminal = Terminal::new(window)?;
//!
//!     loop {
//!         // blocks until the compositor is ready for a new frame
//!         terminal.draw(|frame| {
//!             let block = Block::bordered().title("raclettui");
//!             frame.render_widget(Paragraph::new("Press q to quit").block(block), frame.area());
//!         })?;
//!
//!         for event in events.drain() {
//!             match event {
//!                 WindowEvent::Keyboard(key) if key.is_press() && key.code == KeyCode::Char('q') => return Ok(()),
//!                 WindowEvent::Closed => return Ok(()),
//!                 _ => {}
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Event loop
//!
//! Wayland input is only read while the window waits for the compositor,
//! which happens inside every `Terminal::draw` call. Drawing in a loop
//! therefore renders at the refresh rate of the output and keeps the
//! [`WindowEventQueue`](events::WindowEventQueue) filled. See [`events`].
//!
//! ## Sizing
//!
//! The ratatui grid is as large as the number of cells of the configured
//! font that fit in the surface. A width or height of `0` lets the compositor
//! size the surface, which requires anchoring to both opposite edges. When the
//! compositor resizes the surface, a [`WindowEvent::Resize`](events::WindowEvent::Resize)
//! is emitted and ratatui redraws the whole grid on the next frame.
//!
//! ## Layer shell
//!
//! The following layer-shell types are re-exported to configure windows:
//!
//! - [`Layer`] — background, bottom, top or overlay
//! - [`Anchor`] — screen edges the surface is attached to
//! - [`KeyboardInteractivity`] — whether the surface receives keyboard focus
//!
//! ## Requirements
//!
//! - A Wayland compositor implementing `wlr-layer-shell` (sway, Hyprland,
//!   river, niri, KDE Plasma, Wayfire, labwc, ...). GNOME does not support it.
//! - `libwayland-client` and `libxkbcommon`.
//! - For the wgpu backend, a Vulkan or OpenGL ES capable driver.
//!
//! ## Limitations
//!
//! - Only the first character of a cell's grapheme is drawn.
//! - Underline and crossed out modifiers are ignored, bold and italic are
//!   only supported by the wgpu backend. Reversed and hidden work in both.
//! - No cursor is drawn and key repeat is not emulated.
//! - Fractional / HiDPI scaling is not handled, surfaces render at scale 1.
//!
//! ## Error handling
//!
//! All fallible operations return [`Error`].
//!
//! [Wayland layer shell]: https://wayland.app/protocols/wlr-layer-shell-unstable-v1
//! [ratatui]: https://ratatui.rs

#![warn(missing_docs)]

pub mod builder;
pub mod events;
pub mod colors;
mod cpu;
mod wgpu;
mod wayland;
mod error;

/// Layer shell types used to configure a [`WindowBuilder`].
pub mod layer {
    pub use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
    pub use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;
    pub use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::KeyboardInteractivity;
}

// flattening imports
pub use layer::{Layer, Anchor, KeyboardInteractivity};
pub use builder::WindowBuilder;
pub use error::Error;
pub use cpu::{
    window::CpuWindow,
    buffer::TerminalBuffer,
};
pub use wgpu::{
    window::WgpuWindow,
};
