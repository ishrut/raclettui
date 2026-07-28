pub mod error;
pub mod window;
pub mod events;
mod ratatui_backend;

pub mod layer_shell {
    pub use wayland_protocols_wlr::layer_shell::v1::client::{
        zwlr_layer_shell_v1::Layer,
        zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity},
    };
}

pub use window::WindowBuilder;
pub use layer_shell::{Layer, Anchor, KeyboardInteractivity};
pub use events::*;


