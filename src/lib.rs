pub mod error;
pub mod events;
mod ratatui_backend;
pub mod window;

pub mod layer_shell {
    pub use wayland_protocols_wlr::layer_shell::v1::client::{
        zwlr_layer_shell_v1::Layer,
        zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity},
    };
}

pub use events::*;
pub use layer_shell::{Anchor, KeyboardInteractivity, Layer};
pub use window::WindowBuilder;
