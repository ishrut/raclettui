pub mod builder;
pub mod events;
mod utils;
mod error;
pub mod window;
pub(crate) mod wayland_state;
mod ratatui_backend;

pub mod layer {
    pub use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
    pub use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;
    pub use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::KeyboardInteractivity;
}

// flattening imports
pub use layer::{Layer, Anchor, KeyboardInteractivity};
pub use builder::WindowBuilder;
pub use error::Error;
