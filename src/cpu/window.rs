// use crate::event::{WindowEvent, WindowEventQueue};
use super::wayland::CpuWaylandState;
use crate::builder::WindowBuilder;
use crate::events::WindowEventQueue;
use super::buffer::CpuBuffer;
use crate::colors;
// use crate::buffer::TerminalBuffer;
use wayland_client::EventQueue;


use std::{
    io::{Seek, SeekFrom, Write},
};

use wayland_client::{
    Connection,
};

// Cpu window states
pub struct CpuWindow {
    /// Wayland state
    pub wl_state: CpuWaylandState,
    /// Wayland event queue
    pub wl_event_queue: EventQueue<CpuWaylandState>,
    /// Pixel buffer and associated data to render
    pub buffer: CpuBuffer,
    /// Window transparency
    pub fg_alpha: f32,
    pub bg_alpha: f32,
}

impl WindowBuilder {
    pub fn init_cpu(self) -> CpuWindow {
        // connect to wayland environment
        let conn = Connection::connect_to_env().unwrap();
        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();
        display.get_registry(&qh, ());

        // initialising initial wayland states
        let mut state = CpuWaylandState::new(&self);

        // Wait for globals
        event_queue.roundtrip(&mut state).unwrap();
        event_queue.roundtrip(&mut state).unwrap();

        if !state.is_surface_configured() {
            panic!("src/cpu/window.rs Unable to initialise window, surface not configured");
        }
        let font = self.get_font();

        //initialise buffer
        let mut buffer = CpuBuffer::new(
            state.window_width,
            state.window_height,
            font,
            self.font_size,
            self.fg_alpha,
            self.bg_alpha,
        );
        let color = colors::rgba_premultiplied((255, 255, 255, 255), self.bg_alpha);
        buffer.clear(color);

        // getting variable for first attach
        let (file, surface, wl_buffer) = match (&mut state.file, &state.surface, &state.buffer) {
            (Some(file), Some(surface), Some(wl_buffer)) => (file, surface, wl_buffer),
            _ => panic!("src/cpu/window.rs, Unable to initialise window, wayland states not fully configured")
        };

        file.seek(SeekFrom::Start(0)).unwrap();
        file.write_all(&buffer.inner).unwrap();
        file.flush().unwrap();
        surface.attach(Some(wl_buffer), 0, 0);
        surface.commit();

        state.set_frame_callback(&qh);

        CpuWindow {
         wl_state: state,
         wl_event_queue: event_queue,
         buffer,
         fg_alpha: self.fg_alpha,
         bg_alpha: self.bg_alpha,
        }
    }
}

impl CpuWindow {

    pub fn redraw<F>(&mut self, mut render_callback: F)
    where F: FnMut(&mut CpuBuffer)
    {
        if !self.wl_state.is_redraw() {
            return
        }
        self.wl_state.needs_redraw = false;

        println!("redrawing");

        // getting variables
        let file = self.wl_state.file.as_mut().unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        let surface = self.wl_state.surface.as_ref().unwrap();
        let wl_buffer = self.wl_state.buffer.as_ref().unwrap();

        // notify to redraw whole surface
        surface.damage(0, 0, self.wl_state.window_width as i32, self.wl_state.window_height as i32);

        render_callback(&mut self.buffer);

        // committing the changes
        file.write_all(&self.buffer.inner).unwrap();
        file.flush().unwrap();
        surface.attach(Some(wl_buffer), 0, 0);
        surface.commit();

        // resetting callback
        self.wl_state.set_frame_callback(&self.wl_event_queue.handle());
    }

    // updates the window events, blocking
    pub fn update(&mut self) {
        self.wl_event_queue.blocking_dispatch(&mut self.wl_state).unwrap();
    }

    pub fn get_event_queue(&self) -> WindowEventQueue {
        self.wl_state.events.clone()
    }

    pub fn rows(&self) -> u32 {
        self.buffer.rows()
    }

    pub fn cols(&self) -> u32 {
        self.buffer.cols()
    }

    pub fn width(&self) -> u32 {
        self.wl_state.window_width
    }
    pub fn height(&self) -> u32 {
        self.wl_state.window_height
    }
}
