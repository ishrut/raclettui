/// Errors that can occur during window creation, rendering, and event handling.
#[derive(Debug)]
pub enum Error {
    /// Failed to connect to the Wayland display server.
    WaylandConnectError(wayland_client::ConnectError),
    /// Failed to dispatch or roundtrip the Wayland event queue.
    WaylandDispatchError(wayland_client::DispatchError),
    /// The Wayland surface was never configured by the compositor.
    WaylandSurfaceConfigurationError,
    /// The compositor does not advertise the `zwlr_layer_shell_v1` global.
    WaylandLayerShellError,
    /// The Wayland surface pointer returned null.
    WaylandSurfacePtrNull,
    /// The Wayland display pointer returned null.
    WaylandDisplayPtrNull,
    /// An error occurred while creating or using the GL display/context/surface.
    GlutinError(glutin::error::Error),
    /// No suitable OpenGL config was found for the window surface.
    GlutinDisplayNull,
    /// Fontconfig is not available on this system.
    NoFontConfig,
    /// Failed to find a font via fontconfig.
    FontConfigError(fontconfig::FontconfigError),
    /// Window height is zero (needed for `NonZeroU32`).
    ZeroWindowHeight,
    /// Window width is zero (needed for `NonZeroU32`).
    ZeroWindowWidth,
    /// Swap interval is zero (needed for `NonZeroU32`).
    ZeroSwapInterval,
    /// Failed to set the Wayland frame callback — surface may have been destroyed.
    WaylandFrameCallbackError,
    /// An error from the `beamterm` rasterizer or GPU renderer.
    BeamTermError(beamterm_core::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::WaylandConnectError(e) => write!(f, "{}", e),
            Self::WaylandDispatchError(e) => write!(f, "{}", e),
            Self::WaylandSurfaceConfigurationError => write!(f, "wayland surface not configured"),
            Self::WaylandSurfacePtrNull => write!(f, "wayland surface pointer is null"),
            Self::WaylandDisplayPtrNull => write!(f, "wayland display pointer is null"),
            Self::GlutinError(e) => write!(f, "{}", e),
            Self::GlutinDisplayNull => write!(f, "glutin display is null"),
            Self::ZeroWindowHeight => write!(f, "window height is zero"),
            Self::ZeroWindowWidth => write!(f, "window width is zero"),
            Self::ZeroSwapInterval => write!(f, "swap interval is zero"),
            Self::WaylandFrameCallbackError => {
                write!(f, "unable to set wayland frame callback, surface dropped?")
            }
            Self::BeamTermError(e) => write!(f, "{}", e),
            Self::NoFontConfig => write!(f,"no font config"),
            Self::FontConfigError(e) => write!(f, "{}", e),
            Self::WaylandLayerShellError => write!(f, "Wayland layer shell error"),
        }
    }
}
