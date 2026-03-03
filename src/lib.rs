//! # (raclettui)
//!
//!  !! STILL UNDER DEVELOPMENT BUT USABLE !!
//!
//! A lightweight Wayland layer shell window abstraction implementing the
//! ratatui backend to make terminal style windows.
//! CPU-based and GPU-based (wgpu) rendering backends.
//!
//! This crate provides:
//!
//! - A high-level [`WindowBuilder`] API for constructing Wayland layer shell windows
//! - CPU rendering via [`CpuWindow`]
//! - GPU rendering via [`WgpuWindow`]
//! - Re-exported Wayland layer-shell enums for convenience
//!
//! ## Backends
//!
//! The crate supports two rendering backends:
//!
//! - **CPU backend** — software rendering using shared memory buffers.
//! - **WGPU backend** — hardware-accelerated rendering using `wgpu`.
//!
//! ## Layer Shell Support
//!
//! This crate integrates with the `wlr-layer-shell` protocol,
//! allowing creation of desktop components such as:
//!
//! - Panels
//! - Overlays
//! - Lock screens
//! - Background surfaces
//!
//! The following types are re-exported for convenience to initialise windows:
//!
//! - [`Layer`]
//! - [`Anchor`]
//! - [`KeyboardInteractivity`]
//!
//! ## Example
//!
//! ```no_run
//! use raclettui::{WindowBuilder, Layer, Anchor};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let window = WindowBuilder::new()
//!         .set_layer(Layer::Top)
//!         .set_anchors(vec![Anchor::Top, Anchor::Left, Anchor::Right])
//!         .init_cpu()?; // for cpu rendering backend
//!         //.init_wgpu()?; for gpu rendering backend
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Modules
//!
//! - [`builder`] — Window construction utilities
//! - [`cpu`] — CPU-based rendering backend
//! - [`wgpu`] — GPU-based rendering backend
//! - [`events`] — Event handling types
//!
//! ## Requirements
//!
//! - Wayland compositor
//! - `wlr-layer-shell` protocol support (for layer surfaces)
//!
//! ## Error Handling
//!
//! All fallible operations return [`Error`].
//!

pub mod builder;
pub mod events;
pub mod colors;
mod cpu;
mod wgpu;
mod error;

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
