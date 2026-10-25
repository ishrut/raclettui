/// Errors returned by raclettui.
///
/// The same type is used as the [`Backend::Error`](ratatui_core::backend::Backend::Error)
/// of both [`CpuWindow`](crate::CpuWindow) and [`WgpuWindow`](crate::WgpuWindow).
#[derive(Debug)]
pub enum Error {
    /// Could not connect to the Wayland compositor (is `WAYLAND_DISPLAY` set?).
    WaylandConnectError(wayland_client::ConnectError),
    /// Dispatching Wayland events failed, usually because the connection was lost.
    WaylandDispatchError(wayland_client::DispatchError),
    /// Sending requests to the compositor failed.
    WaylandFlushError(wayland_backend::client::WaylandError),
    /// The compositor does not advertise a global this crate requires,
    /// e.g. `zwlr_layer_shell_v1` on compositors without layer shell support.
    MissingGlobal(&'static str),
    /// The window was used before its surface or buffers were set up.
    WaylandSurfaceConfigurationError,
    /// The [`WindowBuilder`](crate::WindowBuilder) describes a surface the
    /// compositor would reject.
    InvalidConfiguration(&'static str),
    /// The compositor closed the layer surface. The window can no longer be drawn to.
    WindowClosed,
    /// An I/O error, e.g. while writing to the shared memory buffer.
    IoError(std::io::Error),
    /// Requesting a frame callback failed because the surface is gone.
    WaylandFrameCallbackError,
    /// Generic ratatui backend error.
    RatatuiBackendError,
    /// Creating the wgpu surface failed.
    WgpuWindowError,
    /// The font could not be read or parsed, or no monospace font was found.
    FontLoadingError,
    /// Acquiring the next surface texture failed.
    WgpuSurfaceError(wgpu::SurfaceError),
    /// No suitable GPU adapter was found.
    WgpuAdapterError(wgpu::RequestAdapterError),
    /// The GPU device could not be created.
    WgpuRequestDeviceError(wgpu::RequestDeviceError),
    /// The requested backend feature is not supported.
    UnsupportedBackendFeature,
    /// A cell outside of the grid was accessed.
    OutOfBounds,
    /// The font metrics needed to compute the cell size are unavailable.
    CharWidthError,
    /// Preparing text for rendering failed.
    TextRenderPrepareError(glyphon::PrepareError),
    /// Rendering text failed.
    TextRenderError(glyphon::RenderError),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::WaylandConnectError(e) => Some(e),
            Self::WaylandDispatchError(e) => Some(e),
            Self::WaylandFlushError(e) => Some(e),
            Self::IoError(e) => Some(e),
            Self::WgpuSurfaceError(e) => Some(e),
            Self::WgpuAdapterError(e) => Some(e),
            Self::WgpuRequestDeviceError(e) => Some(e),
            Self::TextRenderPrepareError(e) => Some(e),
            Self::TextRenderError(e) => Some(e),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::WaylandConnectError(e) => write!(f, "{}", e),
            Self::WaylandDispatchError(e) => write!(f, "{}", e),
            Self::WaylandFlushError(e) => write!(f, "{}", e),
            Self::MissingGlobal(name) => write!(f, "compositor does not support {}", name),
            Self::WaylandSurfaceConfigurationError => write!(f, "wayland surface not configured"),
            Self::InvalidConfiguration(reason) => write!(f, "invalid window configuration: {}", reason),
            Self::WindowClosed => write!(f, "the compositor closed the window"),
            Self::IoError(e) => write!(f, "{}", e),
            Self::WaylandFrameCallbackError => write!(f, "unable to set wayland frame callback, surface dropped?"),
            Self::RatatuiBackendError => write!(f, "ratatui backend error"),
            Self::WgpuWindowError => write!(f, "error creating wgpu window"),
            Self::FontLoadingError => write!(f, "unable to load font"),
            Self::WgpuSurfaceError(e) => write!(f, "{}", e),
            Self::WgpuAdapterError(e) => write!(f, "{}", e),
            Self::WgpuRequestDeviceError(e) => write!(f, "{}", e),
            Self::UnsupportedBackendFeature => write!(f, "unsupported backend feature"),
            Self::OutOfBounds => write!(f, "setting character out of bounds"),
            Self::CharWidthError => write!(f, "unable to get font metrics to measure cell width"),
            Self::TextRenderPrepareError(e) => write!(f, "{}", e),
            Self::TextRenderError(e) => write!(f, "{}", e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}
