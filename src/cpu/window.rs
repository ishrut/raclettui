use crate::{colors::RaclettuiColor, events::{WindowEvent, WindowEventQueue}};
use super::wayland::CpuWaylandState;
use crate::builder::WindowBuilder;
use super::buffer::TerminalBuffer;
use wayland_client::EventQueue;
use crate::Error;


use std::{
    io::{Seek, SeekFrom, Write},
};

use wayland_client::{
    Connection,
};

// holds data required to draw the window
pub struct CpuWindow {
    pub state: CpuWaylandState,
    pub event_queue: EventQueue<CpuWaylandState>,
    pub buffer: TerminalBuffer,
    pub alpha: f32,
}

impl WindowBuilder {
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

        // Wait for globals
        event_queue.roundtrip(&mut state)
            .map_err(|e| Error::WaylandDispatchError(e))?;
        event_queue.roundtrip(&mut state)
            .map_err(|e| Error::WaylandDispatchError(e))?;

        if !state.is_surface_configured() {
            panic!("wayland layer shell not configured")
        }

        //initialise buffer
        let font = self.get_font();
        let font_size = self.font_size;
        let mut buffer = TerminalBuffer::new(state.window_width, state.window_height, font, font_size);
        buffer.clear(&RaclettuiColor::from_rgba(0, 0, 0, (255.*self.bg_alpha) as u8));

        // getting variable for first attach
        let (file, surface, wl_buffer) =
             match (&mut state.file, &state.surface, &state.buffer) {
                (Some(file), Some(surface), Some(wl_buffer)) => (file, surface, wl_buffer),
                _ => return Err(Error::WaylandSurfaceConfigurationError)
            };

        file.seek(SeekFrom::Start(0))
            .map_err(|e| Error::IoError(e))?;
        file.write_all(&buffer.inner)
            .map_err(|e| Error::IoError(e))?;
        file.flush()
            .map_err(|e| Error::IoError(e))?;
        surface.attach(Some(wl_buffer), 0, 0);
        surface.commit();

        state.frame_callback = Some(surface.frame(&qh, ()));

        Ok(CpuWindow {
            state,
            event_queue,
            buffer,
            alpha: self.bg_alpha
        })
    }
}

impl CpuWindow {

    // testing purposes function, draw to the window provided with a callback function
    // with the buffer as argument
    pub fn redraw<F>(&mut self, mut render_callback: F)
    where F: FnMut(&mut TerminalBuffer)
    {
        if !self.state.needs_redraw {
            return
        }
        self.state.needs_redraw = false;

        // getting variables
        let (file, surface, wl_buffer) =
            match (&mut self.state.file, &self.state.surface, &self.state.buffer) {
                (Some(file), Some(surface), Some(buffer)) => (file, surface, buffer),
                _ => panic!("not configured")
            };
        file.seek(SeekFrom::Start(0)).unwrap();

        // notify to redraw whole surface
        surface.damage(0, 0, self.state.window_width as i32, self.state.window_height as i32);

        render_callback(&mut self.buffer);

        // committing the changes
        file.write_all(&self.buffer.inner).unwrap();
        file.flush().unwrap();
        surface.attach(Some(wl_buffer), 0, 0);
        surface.commit();

        // resetting callback
        self.state.frame_callback = Some(surface.frame(&self.event_queue.handle(), ()));
    }

    // updates the window events
    pub fn update(&mut self) -> Result<(), Error> {
        self.event_queue.roundtrip(&mut self.state)
            .map_err(|e| Error::WaylandDispatchError(e))?;
        Ok(())
    }

    pub fn get_event_queue(&self) -> WindowEventQueue {
        self.state.events.clone()
    }
}


