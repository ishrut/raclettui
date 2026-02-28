
#[derive(Debug)]
pub enum Error {
    WaylandConnectError(wayland_client::ConnectError),
    WaylandDispatchError(wayland_client::DispatchError),
    WaylandSurfaceConfigurationError,
    IoError(std::io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::WaylandConnectError(e) => write!(f, "{}", e),
            Self::WaylandDispatchError(e) => write!(f, "{}", e),
            Self::WaylandSurfaceConfigurationError => write!(f, "wayland surface not configured"),
            Self::IoError(e) => write!(f, "{}", e)
        }
    }
}
