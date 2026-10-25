use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::io::AsFd;

use ratatui_core::layout::Position;
use wayland_client::{
    Connection, EventQueue, QueueHandle,
    protocol::{wl_buffer, wl_shm, wl_shm_pool},
};

use super::buffer::TerminalBuffer;
use crate::builder::WindowBuilder;
use crate::colors::RaclettuiColor;
use crate::events::{WindowEvent, WindowEventQueue};
use crate::wayland::{self, WaylandState};
use crate::Error;

/// A layer-shell window rendered on the CPU.
///
/// Cells are rasterized into a [`TerminalBuffer`] and presented through a
/// Wayland shared memory buffer, so no GPU is required. Create it with
/// [`WindowBuilder::init_cpu`] and pass it to [`ratatui::Terminal::new`]:
///
/// ```no_run
/// use raclettui::{WindowBuilder, Anchor};
/// use ratatui::{Terminal, widgets::Paragraph};
///
/// let window = WindowBuilder::new()
///     .set_width(400)
///     .set_height(200)
///     .set_anchor(Anchor::Top)
///     .init_cpu()?;
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
pub struct CpuWindow {
    pub(crate) buffer: TerminalBuffer,
    pub(crate) bg_alpha: f32,
    pub(crate) fg_alpha: f32,
    pub(crate) cursor: Position,

    // wayland objects are dropped last
    shm_buffer: ShmBuffer,
    pub(crate) state: WaylandState,
    event_queue: EventQueue<WaylandState>,
    conn: Connection,
}

/// A `wl_buffer` backed by a temporary file.
struct ShmBuffer {
    file: File,
    pool: wl_shm_pool::WlShmPool,
    buffer: wl_buffer::WlBuffer,
}

impl ShmBuffer {
    fn new(
        shm: &wl_shm::WlShm,
        qh: &QueueHandle<WaylandState>,
        width: u32,
        height: u32,
    ) -> Result<Self, Error> {
        let stride = width.checked_mul(4).ok_or(Error::OutOfBounds)?;
        let size = stride
            .checked_mul(height)
            .and_then(|size| i32::try_from(size).ok())
            .ok_or(Error::OutOfBounds)?;

        let file = tempfile::tempfile()?;
        file.set_len(size as u64)?;

        let pool = shm.create_pool(file.as_fd(), size, qh, ());
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            wl_shm::Format::Argb8888,
            qh,
            (),
        );
        Ok(Self { file, pool, buffer })
    }
}

impl Drop for ShmBuffer {
    fn drop(&mut self) {
        self.buffer.destroy();
        self.pool.destroy();
    }
}

impl WindowBuilder {
    /// Creates a window using the CPU rendering backend.
    ///
    /// Connects to the compositor, creates the layer surface and waits for
    /// it to be configured.
    ///
    /// # Errors
    ///
    /// - [`Error::WaylandConnectError`] if no Wayland compositor is running.
    /// - [`Error::MissingGlobal`] if the compositor lacks layer shell or shm support.
    /// - [`Error::InvalidConfiguration`] for an invalid size / anchor combination.
    /// - [`Error::IoError`] or [`Error::FontLoadingError`] if the font cannot be loaded.
    pub fn init_cpu(self) -> Result<CpuWindow, Error> {
        let font = self.get_font()?;
        let (conn, event_queue, state) = wayland::connect(&self)?;
        let shm = state.shm.as_ref().ok_or(Error::MissingGlobal("wl_shm"))?;
        let shm_buffer = ShmBuffer::new(shm, &event_queue.handle(), state.width, state.height)?;

        let mut buffer = TerminalBuffer::new(state.width, state.height, font, self.font_size);
        let window_bg = RaclettuiColor::DEFAULT_BG.premultiply_alpha(self.bg_alpha);
        buffer.clear(&window_bg);

        Ok(CpuWindow {
            buffer,
            bg_alpha: self.bg_alpha,
            fg_alpha: self.fg_alpha,
            cursor: Position::ORIGIN,
            shm_buffer,
            state,
            event_queue,
            conn,
        })
    }
}

impl CpuWindow {
    /// Draws directly into the pixel buffer with `render_callback` and
    /// presents the result, without going through ratatui.
    ///
    /// Blocks until the compositor is ready for a new frame.
    pub fn redraw<F>(&mut self, render_callback: F) -> Result<(), Error>
    where F: FnOnce(&mut TerminalBuffer)
    {
        render_callback(&mut self.buffer);
        self.present()
    }

    /// Blocks until at least one Wayland event was received and dispatches it.
    ///
    /// Input events are pushed to the [`WindowEventQueue`]. This is only needed
    /// when not drawing, as drawing already dispatches events.
    pub fn update(&mut self) -> Result<(), Error> {
        self.event_queue.blocking_dispatch(&mut self.state)
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
        self.buffer.window_width
    }

    /// Window height in pixels.
    pub fn height(&self) -> u32 {
        self.buffer.window_height
    }

    /// The pixel buffer the window is rendered from.
    pub fn buffer(&self) -> &TerminalBuffer {
        &self.buffer
    }

    /// Color the window is cleared with: the default background at the configured opacity.
    pub(crate) fn window_bg(&self) -> RaclettuiColor {
        RaclettuiColor::DEFAULT_BG.premultiply_alpha(self.bg_alpha)
    }

    /// Waits for the next frame and presents the pixel buffer.
    ///
    /// If the compositor resized the surface in the meantime, the buffers are
    /// reallocated and cleared and a [`WindowEvent::Resize`] is emitted, ratatui
    /// then redraws everything on its next draw.
    pub(crate) fn present(&mut self) -> Result<(), Error> {
        wayland::wait_for_frame(&mut self.event_queue, &mut self.state)?;

        let (width, height) = (self.state.width, self.state.height);
        if (width, height) != self.buffer.window_size() {
            let shm = self.state.shm.as_ref().ok_or(Error::MissingGlobal("wl_shm"))?;
            self.shm_buffer = ShmBuffer::new(shm, &self.event_queue.handle(), width, height)?;
            self.buffer.resize(width, height);
            self.buffer.clear(&self.window_bg());
            self.state.events.push(WindowEvent::new_resize_event(width, height));
        }

        let file = &mut self.shm_buffer.file;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(&self.buffer.inner)?;
        file.flush()?;

        let qh = self.event_queue.handle();
        self.state.request_frame(&qh)?;
        let surface = self.state.surface.as_ref().ok_or(Error::WaylandSurfaceConfigurationError)?;
        surface.attach(Some(&self.shm_buffer.buffer), 0, 0);
        surface.damage(0, 0, width as i32, height as i32);
        surface.commit();
        self.conn.flush().map_err(Error::WaylandFlushError)?;
        Ok(())
    }
}
