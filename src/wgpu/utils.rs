use pollster::FutureExt as _;
use crate::Error;

pub fn wgpu_create_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> Result<wgpu::Adapter, Error> {

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
        })
        .block_on()
        .map_err(Error::WgpuAdapterError)?;
    Ok(adapter)
}

pub fn wgpu_get_device_queue(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue), Error> {

    let (wgpu_device, wgpu_queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("Default Wgpu Device"),
            required_features: wgpu::Features::empty(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: wgpu::Limits::default(),
            // whether to favor performance or memory, performance is default
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        })
        .block_on()
        .map_err(Error::WgpuRequestDeviceError)?;

    Ok((wgpu_device, wgpu_queue))
}

// if window is resized new configuration is needed
pub fn wgpu_get_surface_config(surface: &wgpu::Surface, adapter: &wgpu::Adapter, width: u32, height: u32) -> Result<wgpu::SurfaceConfiguration, Error> {

    let surface_caps = surface.get_capabilities(adapter);

    // an empty capability list means the adapter cannot present to this surface
    let first_format = *surface_caps.formats.first().ok_or(Error::WgpuWindowError)?;
    let surface_format = surface_caps
        .formats
        .iter()
        // colors are converted to linear space, so an sRGB format is needed
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(first_format);

    // premultiplied alpha is needed for the window transparency
    let alpha_mode = if surface_caps
        .alpha_modes
        .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
    {
        wgpu::CompositeAlphaMode::PreMultiplied
    } else {
        surface_caps.alpha_modes.first().copied().unwrap_or(wgpu::CompositeAlphaMode::Auto)
    };
    let present_mode = surface_caps
        .present_modes
        .first()
        .copied()
        .unwrap_or(wgpu::PresentMode::Fifo);

    let wgpu_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width,
        height,
        present_mode,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    Ok(wgpu_config)
}
