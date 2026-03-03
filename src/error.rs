
#[derive(Debug)]
pub enum Error {
    WaylandConnectError(wayland_client::ConnectError),
    WaylandDispatchError(wayland_client::DispatchError),
    WaylandSurfaceConfigurationError,
    IoError(std::io::Error),
    WaylandFrameCallbackError,
    RatatuiBackendError,
    WgpuWindowError,
    FontLoadingError,
    WgpuSurfaceError(wgpu::SurfaceError),
    WgpuAdapterError(wgpu::RequestAdapterError),
    WgpuRequestDeviceError(wgpu::RequestDeviceError),
    UnsupportedBackendFeature,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::WaylandConnectError(e) => write!(f, "{}", e),
            Self::WaylandDispatchError(e) => write!(f, "{}", e),
            Self::WaylandSurfaceConfigurationError => write!(f, "wayland surface not configured"),
            Self::IoError(e) => write!(f, "{}", e),
            Self::WaylandFrameCallbackError => write!(f, "unable to set wayland frame callback, surface dropped?"),
            Self::RatatuiBackendError => write!(f, "ratatui backend error"),
            Self::WgpuWindowError => write!(f, "error creating wgpu window"),
            Self::FontLoadingError => write!(f, "unable to load font"),
            Self::WgpuSurfaceError(e) => write!(f, "{}", e),
            Self::WgpuAdapterError(e) => write!(f, "{}", e),
            Self::WgpuRequestDeviceError(e) => write!(f, "{}", e),
            Self::UnsupportedBackendFeature => write!(f, "unsupported backend feature"),
        }
    }
}
