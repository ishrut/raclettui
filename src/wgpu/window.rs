use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use std::ffi::c_void;

use crate::WindowBuilder;
use super::wayland::WgpuWaylandState;
use super::utils;
use crate::events::WindowEventQueue;
use super::render::GridRenderer;
use wayland_client::{Connection, EventQueue, Proxy};

pub struct WgpuWindow {

    pub wgpu_surface: wgpu::Surface<'static>,
    pub wgpu_device: wgpu::Device,
    pub wgpu_config: wgpu::SurfaceConfiguration,
    pub wgpu_queue: wgpu::Queue,

    pub grid_renderer: GridRenderer,

    // wayland objects to be dropped last to prevent segfault
    pub wayland_state: WgpuWaylandState,
    pub wayland_event_queue: EventQueue<WgpuWaylandState>,

    pub bg_alpha: f32,
}

impl WindowBuilder {
    pub fn init_wgpu(self) -> WgpuWindow {
        let conn = Connection::connect_to_env().unwrap();
        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();

        let display = conn.display();
        display.get_registry(&qh, ());

        let mut state = WgpuWaylandState::new(&self);

        event_queue.roundtrip(&mut state).unwrap();
        event_queue.roundtrip(&mut state).unwrap();

        if !state.is_surface_configured() {
            panic!("layer shell not configured")
        }
        event_queue.roundtrip(&mut state).unwrap();


        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
        });

        let backend = conn.backend();
        let wl_display_ptr = backend.display_ptr() as *mut std::ffi::c_void;
        let display_ptr = unsafe { core::ptr::NonNull::new_unchecked(wl_display_ptr) };

        let surface_ptr = state.surface.as_ref().unwrap().id().as_ptr() as *mut c_void;
        let surface_ptr = unsafe { core::ptr::NonNull::new_unchecked(surface_ptr) };

        let wgpu_surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                    display_ptr,
                )),
                raw_window_handle: RawWindowHandle::Wayland(WaylandWindowHandle::new(surface_ptr)),
            })
        }
        .unwrap();

        let adapter = utils::wgpu_create_adapter(&instance, &wgpu_surface);
        let (wgpu_device, wgpu_queue) = utils::wgpu_get_device_queue(&adapter);
        let wgpu_config = utils::wgpu_get_surface_config(&wgpu_surface, &adapter, state.window_width, state.window_height);
        wgpu_surface.configure(&wgpu_device, &wgpu_config);

        let font = self.get_font();

        let grid_renderer = GridRenderer::new(
            font,
            &wgpu_config,
            &wgpu_device,
            &self,
        );

        state.set_frame_callback(&qh);

        WgpuWindow {
            wayland_event_queue: event_queue,
            wayland_state: state,
            wgpu_config: wgpu_config,
            wgpu_surface,
            wgpu_device,
            wgpu_queue,
            bg_alpha: self.bg_alpha,
            grid_renderer,
        }
    }
}

impl WgpuWindow {
    pub fn update(&mut self) {
        self.wayland_event_queue
            .blocking_dispatch(&mut self.wayland_state)
            .unwrap();
    }

    pub fn redraw<F>(&mut self, mut render_callback: F)
    where
        F: FnMut(),
    {
        // checks if redraw requested
        if !self.wayland_state.needs_redraw {
            return;
        }
        self.wayland_state.needs_redraw = false;

        //render callback
        render_callback();

        // resets a frame callback
        self.wayland_state.set_frame_callback(&self.wayland_event_queue.handle());
    }

    pub fn is_redraw(&mut self) -> bool {
        if !self.wayland_state.needs_redraw {
            return false
        } else {
            self.wayland_state.needs_redraw = false;
            self.wayland_state.set_frame_callback(&self.wayland_event_queue.handle());
            return true
        }

    }

    pub fn clear_screen(&self) {
        let output = self.wgpu_surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.wgpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Clear Screen Render Encoder"),
        });
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: self.bg_alpha as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }
        self.wgpu_queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn get_event_queue(&self) -> WindowEventQueue {
        self.wayland_state.events.clone()
    }

    pub fn width(&self) -> u32 {
        self.wayland_state.window_width
    }
    pub fn height(&self) -> u32 {
        self.wayland_state.window_height
    }

    pub fn render(
        &mut self,
    ) {

        let output = self.wgpu_surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.wgpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear({
                            wgpu::Color { r: 0., g: 0., b: 0., a: self.bg_alpha as f64 }
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
            self.grid_renderer.render_text(&self.wgpu_device, &self.wgpu_queue, &mut render_pass);
        }
        self.wgpu_queue.submit(vec![encoder.finish()]);
        output.present();

    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub ch: char,
    pub fg_color: (u8, u8, u8),
    pub bg_color: (u8, u8, u8),
}

impl Cell {
    pub fn new(ch: char, fg: (u8, u8, u8), bg: (u8, u8, u8)) -> Self {
        Self {ch, fg_color: fg, bg_color: bg }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            ch: ' ',
            fg_color: (255, 255, 255),
            bg_color: (0, 0, 0),
        }
    }
}

pub struct Grid {
    cells: Vec<Vec<Cell>>,
    cols: usize,
    rows: usize,
}

impl Grid {

    pub fn new(rows: usize, cols: usize) -> Self {
        let mut cells = Vec::new();
        for _i in 0..rows {
            let mut row = Vec::new();
            for _j in 0..cols {
                row.push(Cell::default());
            }
            cells.push(row);
        }
        Grid {
            cells,
            cols,
            rows,
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        // If size hasn't changed, nothing to do
        if cols == self.cols && rows == self.rows {
            return;
        }

        let mut new_cells = Vec::new();
        for _i in 0..rows {
            let mut row = Vec::new();
            for _j in 0..cols {
                row.push(Cell::default());
            }
            new_cells.push(row);
        }

        // Copy overlapping content from old buffer to new buffer
        let copy_rows = self.rows.min(rows);
        let copy_cols = self.cols.min(cols);

        for row in 0..copy_rows {
            for col in 0..copy_cols {
                new_cells[row][col] = self.cells[row][col]
            }
        }

        // Update grid state
        self.cols = cols;
        self.rows = rows;
        self.cells = new_cells;

    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&Cell> {
        match self.cells.get(row) {
            Some(row_v) => {
                match row_v.get(col) {
                    Some(v) => Some(v),
                    None => None
                }
            }
            None => None
        }
    }

    pub fn set_cell(
        &mut self,
         row: usize,
         col: usize,
         ch: char,
         fg: (u8, u8, u8),
         bg: (u8, u8, u8),
     )
     {
         let cell = Cell {
             ch,
             fg_color: fg,
             bg_color: bg,
         };

        if col < self.cols && row < self.rows {
            self.cells[row][col] = cell
        }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }
}
