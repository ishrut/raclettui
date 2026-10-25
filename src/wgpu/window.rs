use std::ffi::c_void;
use std::ptr::NonNull;

use ratatui_core::layout::Position;
use ratatui_core::style::Modifier;
use wayland_client::{Connection, EventQueue, Proxy};
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};

use super::render::GridRenderer;
use super::utils;
use crate::colors::RaclettuiColor;
use crate::events::{WindowEvent, WindowEventQueue};
use crate::wayland::{self, WaylandState};
use crate::Error;
use crate::WindowBuilder;

/// A layer-shell window rendered on the GPU with `wgpu`.
///
/// Cell backgrounds are drawn as quads and text is rendered with `glyphon`.
/// Create it with [`WindowBuilder::init_wgpu`] and pass it to
/// [`ratatui::Terminal::new`]:
///
/// ```no_run
/// use raclettui::{WindowBuilder, Anchor};
/// use ratatui::{Terminal, widgets::Paragraph};
///
/// let window = WindowBuilder::new()
///     .set_width(400)
///     .set_height(200)
///     .set_anchor(Anchor::Top)
///     .init_wgpu()?;
/// let mut terminal = Terminal::new(window)?;
/// terminal.draw(|f| f.render_widget(Paragraph::new("Hello"), f.area()))?;
/// # Ok::<(), raclettui::Error>(())
/// ```
///
/// Each `Terminal::draw` blocks until the compositor is ready for a new
/// frame, which paces the application to the refresh rate of the output
/// and dispatches pending input events.
///
/// [`ratatui::Terminal::new`]: https://docs.rs/ratatui/latest/ratatui/struct.Terminal.html#method.new
pub struct WgpuWindow {
    // fields are dropped in declaration order: gpu resources first, then the
    // wgpu surface and finally the wayland objects it points to
    pub(crate) grid_renderer: GridRenderer,
    wgpu_surface: wgpu::Surface<'static>,
    wgpu_device: wgpu::Device,
    wgpu_queue: wgpu::Queue,
    wgpu_config: wgpu::SurfaceConfiguration,
    pub(crate) cursor: Position,

    pub(crate) state: WaylandState,
    event_queue: EventQueue<WaylandState>,
    _conn: Connection,
}

impl WindowBuilder {
    /// Creates a window using the wgpu rendering backend.
    ///
    /// Connects to the compositor, creates the layer surface, waits for it
    /// to be configured and sets up a wgpu device rendering to it.
    ///
    /// # Errors
    ///
    /// - [`Error::WaylandConnectError`] if no Wayland compositor is running.
    /// - [`Error::MissingGlobal`] if the compositor lacks layer shell support.
    /// - [`Error::InvalidConfiguration`] for an invalid size / anchor combination.
    /// - [`Error::WgpuAdapterError`] / [`Error::WgpuRequestDeviceError`] if no usable GPU is found.
    /// - [`Error::IoError`] or [`Error::FontLoadingError`] if the font cannot be loaded.
    pub fn init_wgpu(self) -> Result<WgpuWindow, Error> {
        let font_system = self.get_font_system()?;
        let (conn, event_queue, state) = wayland::connect(&self)?;

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::default(),
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
        });

        let display_ptr = NonNull::new(conn.backend().display_ptr() as *mut c_void)
            .ok_or(Error::WgpuWindowError)?;
        let surface = state.surface.as_ref().ok_or(Error::WaylandSurfaceConfigurationError)?;
        let surface_ptr = NonNull::new(surface.id().as_ptr() as *mut c_void)
            .ok_or(Error::WgpuWindowError)?;

        // SAFETY: the display and surface outlive the wgpu surface, which is
        // dropped first (see the field order of `WgpuWindow`).
        let wgpu_surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display_ptr)),
                raw_window_handle: RawWindowHandle::Wayland(WaylandWindowHandle::new(surface_ptr)),
            })
        }
        .map_err(|_| Error::WgpuWindowError)?;

        let adapter = utils::wgpu_create_adapter(&instance, &wgpu_surface)?;
        let (wgpu_device, wgpu_queue) = utils::wgpu_get_device_queue(&adapter)?;
        let wgpu_config = utils::wgpu_get_surface_config(&wgpu_surface, &adapter, state.width, state.height)?;
        wgpu_surface.configure(&wgpu_device, &wgpu_config);

        let grid_renderer = GridRenderer::new(
            font_system,
            &wgpu_config,
            &wgpu_device,
            &wgpu_queue,
            &self,
        )?;

        Ok(WgpuWindow {
            grid_renderer,
            wgpu_surface,
            wgpu_device,
            wgpu_queue,
            wgpu_config,
            cursor: Position::ORIGIN,
            state,
            event_queue,
            _conn: conn,
        })
    }
}

impl WgpuWindow {
    /// Blocks until at least one Wayland event was received and dispatches it.
    ///
    /// Input events are pushed to the [`WindowEventQueue`]. This is only needed
    /// when not drawing, as drawing already dispatches events.
    pub fn update(&mut self) -> Result<(), Error> {
        self.event_queue
            .blocking_dispatch(&mut self.state)
            .map_err(Error::WaylandDispatchError)?;
        Ok(())
    }

    /// Returns a handle to the queue receiving this window's input events.
    ///
    /// Call this before moving the window into a ratatui `Terminal`.
    pub fn get_event_queue(&self) -> WindowEventQueue {
        self.state.events.clone()
    }

    /// Window width in pixels.
    pub fn width(&self) -> u32 {
        self.wgpu_config.width
    }

    /// Window height in pixels.
    pub fn height(&self) -> u32 {
        self.wgpu_config.height
    }

    /// Number of `(columns, rows)` in the cell grid.
    pub fn grid_size(&self) -> (u32, u32) {
        (self.grid_renderer.grid.cols, self.grid_renderer.grid.rows)
    }

    /// Sets the character and colors of the cell at column `x`, row `y`,
    /// without going through ratatui. Call [`render`](Self::render) to show it.
    ///
    /// Colors are used as is, without applying the window's alpha settings.
    pub fn set_cell(
        &mut self,
        x: u32,
        y: u32,
        ch: char,
        fg: RaclettuiColor,
        bg: RaclettuiColor,
    ) -> Result<(), Error> {
        let grid = &mut self.grid_renderer.grid;
        grid.set_bg(y, x, bg)?;
        grid.set_ch(y, x, ch, fg, Modifier::empty(), &mut self.grid_renderer.font_system)
    }

    /// Waits until the compositor is ready for a new frame and renders the grid.
    ///
    /// If the compositor resized the surface in the meantime, the grid is
    /// resized and cleared and a [`WindowEvent::Resize`] is emitted, ratatui
    /// then redraws everything on its next draw.
    pub fn render(&mut self) -> Result<(), Error> {
        wayland::wait_for_frame(&mut self.event_queue, &mut self.state)?;

        let (width, height) = (self.state.width, self.state.height);
        if (width, height) != (self.wgpu_config.width, self.wgpu_config.height) {
            self.wgpu_config.width = width;
            self.wgpu_config.height = height;
            self.wgpu_surface.configure(&self.wgpu_device, &self.wgpu_config);
            self.grid_renderer.resize(width, height);
            self.state.events.push(WindowEvent::new_resize_event(width, height));
        }

        let output = match self.wgpu_surface.get_current_texture() {
            Ok(output) => output,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                self.wgpu_surface.configure(&self.wgpu_device, &self.wgpu_config);
                self.wgpu_surface
                    .get_current_texture()
                    .map_err(Error::WgpuSurfaceError)?
            }
            Err(wgpu::SurfaceError::Timeout) => {
                // nothing was committed so no frame callback will fire, retry on the next frame
                self.state.needs_redraw = true;
                return Ok(());
            }
            Err(e) => return Err(Error::WgpuSurfaceError(e)),
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.wgpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let window_bg = RaclettuiColor::DEFAULT_BG
                .set_alpha(self.grid_renderer.grid.bg_alpha)
                .to_linear();
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: window_bg[0] as f64,
                            g: window_bg[1] as f64,
                            b: window_bg[2] as f64,
                            a: window_bg[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.grid_renderer.render_background(&self.wgpu_device, &mut render_pass);
            self.grid_renderer.render_text(&self.wgpu_queue, &self.wgpu_device, &mut render_pass)?;
        }
        self.wgpu_queue.submit(std::iter::once(encoder.finish()));

        // the frame callback must be requested before present() commits the surface
        let qh = self.event_queue.handle();
        self.state.request_frame(&qh)?;
        output.present();
        Ok(())
    }
}
