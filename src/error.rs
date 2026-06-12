#[derive(Debug)]
pub enum Error {
    WaylandConnectError(wayland_client::ConnectError),
    WaylandDispatchError(wayland_client::DispatchError),
    WaylandSurfaceConfigurationError,
    WaylandSurfacePtrNull,
    WaylandDisplayPtrNull,
    IoError(std::io::Error),
    WaylandFrameCallbackError,
    RatatuiBackendError,
    FontLoadingError,
    UnsupportedBackendFeature,
    OutOfBounds,
    CharWidthError,
    GlutinError(glutin::error::Error),
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
            Self::IoError(e) => write!(f, "{}", e),
            Self::WaylandFrameCallbackError => {
                write!(f, "unable to set wayland frame callback, surface dropped?")
            }
            Self::RatatuiBackendError => write!(f, "ratatui backend error"),
            Self::FontLoadingError => write!(f, "unable to load font"),
            Self::UnsupportedBackendFeature => write!(f, "unsupported backend feature"),
            Self::OutOfBounds => write!(f, "setting character out of bounds"),
            Self::CharWidthError => write!(f, "unable to get font metrics to measure cell width"),
            Self::GlutinError(e) => write!(f, "{}", e),
            Self::BeamTermError(e) => write!(f, "{}", e),
        }
    }
}
