use std::io::{Seek, SeekFrom, Write};
use wayland_client::{Connection, EventQueue};

use super::wayland::CpuWaylandState;
use super::buffer::CpuBuffer;
use crate::builder::WindowBuilder;
use crate::events::WindowEventQueue;
use crate::colors;
use crate::Error;

// Cpu window states
pub struct CpuWindow {
    /// Wayland state
    pub wl_state: CpuWaylandState,
    /// Wayland event queue
    pub wl_event_queue: EventQueue<CpuWaylandState>,
    /// Pixel buffer and associated data to render
    pub buffer: CpuBuffer,
    /// Window transparency
    pub bg_alpha: f32,
    pub fg_alpha: f32,
}

impl WindowBuilder {

    /// Initialises cpu window
    pub fn init_cpu(self) -> Result<CpuWindow, Error> {
        // connect to wayland environment
        let conn = Connection::connect_to_env()
            .map_err(|e| Error::WaylandConnectError(e))?;

        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();
        display.get_registry(&qh, ());

        // initialising initial wayland states
        let mut state = CpuWaylandState::new(&self);

        // wait for globals
        event_queue.roundtrip(&mut state)
            .map_err(|e| Error::WaylandDispatchError(e))?;
        event_queue.roundtrip(&mut state)
            .map_err(|e| Error::WaylandDispatchError(e))?;

        // checks if state is configured
        if !state.is_surface_configured() {
            return Err(Error::WaylandSurfaceConfigurationError)
        }

        // loads font
        let font = self.get_font();

        // initialise buffer
        let mut buffer = CpuBuffer::new(
            state.window_width,
            state.window_height,
            font,
            self.font_size,
            self.fg_alpha,
            self.bg_alpha,
        );

        // clears screen with black
        let color = colors::rgba_premultiplied((0, 0, 0, 0), self.bg_alpha);
        buffer.clear(color);

        // getting variable for first attach
        let (file, surface, wl_buffer) =
            match (&mut state.file, &state.surface, &state.buffer) {
                (Some(file), Some(surface), Some(wl_buffer)) => (file, surface, wl_buffer),
                _ => return Err(Error::WaylandSurfaceConfigurationError)
        };

        // writing to buffer
        file.seek(SeekFrom::Start(0))
            .map_err(|e| Error::IoError(e))?;
        file.write_all(&buffer.inner)
            .map_err(|e| Error::IoError(e))?;
        file.flush()
            .map_err(|e| Error::IoError(e))?;
        surface.attach(Some(wl_buffer), 0, 0);
        surface.commit();

        // set frame callback
        state.set_frame_callback(&qh);

        Ok(CpuWindow {
         wl_state: state,
         wl_event_queue: event_queue,
         buffer,
         fg_alpha: self.fg_alpha,
         bg_alpha: self.bg_alpha,
        })
    }
}

impl CpuWindow {

    /// Redraws window with a callback and gives the buffer as argument
    pub fn redraw<F>(&mut self, mut render_callback: F) -> Result<(), Error>
    where F: FnMut(&mut CpuBuffer)
    {
        // checking redraw condition
        if !self.wl_state.is_redraw() {
            return Ok(())
        }
        self.wl_state.needs_redraw = false;

        // getting variables
        let (file, surface, wl_buffer) =
            match (&mut self.wl_state.file, &self.wl_state.surface, &self.wl_state.buffer) {
                (Some(file), Some(surface), Some(wl_buffer)) => (file, surface, wl_buffer),
                _ => return Err(Error::WaylandSurfaceConfigurationError)
        };

        // notify to redraw whole surface
        surface.damage(0, 0, self.wl_state.window_width as i32, self.wl_state.window_height as i32);

        // callback to modify buffer
        render_callback(&mut self.buffer);

        //writing to buffer
        file.seek(SeekFrom::Start(0))
            .map_err(|e| Error::IoError(e))?;
        file.write_all(&self.buffer.inner)
            .map_err(|e| Error::IoError(e))?;
        file.flush()
            .map_err(|e| Error::IoError(e))?;
        surface.attach(Some(wl_buffer), 0, 0);
        surface.commit();

        // resetting callback
        self.wl_state.set_frame_callback(&self.wl_event_queue.handle());
        Ok(())
    }

    /// Updates the window events, blocking
    pub fn update(&mut self) {
        self.wl_event_queue.blocking_dispatch(&mut self.wl_state).unwrap();
    }

    /// Get event queue
    pub fn get_event_queue(&self) -> WindowEventQueue {
        self.wl_state.events.clone()
    }

    /// Number of rows
    pub fn rows(&self) -> u32 {
        self.buffer.rows()
    }

    /// Number of columns
    pub fn cols(&self) -> u32 {
        self.buffer.cols()
    }

    /// Window width in px
    pub fn width(&self) -> u32 {
        self.wl_state.window_width
    }

    /// Window height in px
    pub fn height(&self) -> u32 {
        self.wl_state.window_height
    }
}
