use crate::events::WindowEventQueue;
use crate::Error;
use crate::{wayland_state::WaylandState, WindowBuilder};
use beamterm_core::{FontAtlasData, GlState, GlslVersion, StaticFontAtlas, TerminalGrid};
use glutin::config::{ConfigTemplateBuilder};
use glutin::context::NotCurrentGlContext;
use glutin::display::GlDisplay;
use glutin::prelude::GlSurface;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use std::ffi::c_void;
use std::num::NonZeroU32;
use wayland_client::{Connection, Proxy};

pub struct Window {
    wayland_event_queue: wayland_client::EventQueue<WaylandState>,
    wayland_state: WaylandState,
    pub(crate) grid: TerminalGrid,
    pub(crate) gl: glow::Context,
    pub(crate) gl_state: GlState,
    pub(crate) gl_context: glutin::context::PossiblyCurrentContext,
    pub(crate) gl_surface: glutin::surface::Surface<WindowSurface>,
}

impl WindowBuilder {
    pub fn init(&self) -> Result<Window, Error> {
        let conn = Connection::connect_to_env()
            .map_err(|e| Error::WaylandConnectError(e))?;
        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();

        let display = conn.display();
        display.get_registry(&qh, ());

        let mut state = WaylandState::new(&self);

        event_queue
            .roundtrip(&mut state)
            .map_err(|e| Error::WaylandDispatchError(e))?;
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| Error::WaylandDispatchError(e))?;

        if !state.is_surface_configured() {
            return Err(Error::WaylandSurfaceConfigurationError);
        }

        let backend = conn.backend();
        let wl_display_ptr = backend.display_ptr() as *mut std::ffi::c_void;
        if wl_display_ptr.is_null() {
            return Err(Error::WaylandDisplayPtrNull);
        }
        let display_ptr = unsafe {
            core::ptr::NonNull::new_unchecked(wl_display_ptr)
        };

        let raw_display_handle = RawDisplayHandle::Wayland(
            WaylandDisplayHandle::new(display_ptr)
        );
        let gl_display = unsafe {
            glutin::display::Display::new(
                raw_display_handle,
                glutin::display::DisplayApiPreference::Egl,
            )
            .map_err(|e| Error::GlutinError(e))?
        };

        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_depth_size(24)
            .build();

        let gl_config = unsafe {
            gl_display
                .find_configs(template)
                .map_err(|e| Error::GlutinError(e))?
                .next()
                .expect("No GL configs found")
        };

        let surface_ptr = state
            .surface()
            .ok_or(Error::WaylandSurfaceConfigurationError)?
            .id()
            .as_ptr() as *mut c_void;
        if surface_ptr.is_null() {
            return Err(Error::WaylandSurfacePtrNull);
        }
        let nonnull_surface_ptr = unsafe { core::ptr::NonNull::new_unchecked(surface_ptr) };
        // let gl_surface_raw = glutin::surface::RawSurface::Egl(surface_ptr);

        let window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(nonnull_surface_ptr));

        let context_attrs = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::OpenGl(Some(
                glutin::context::Version::new(3, 3),
            )))
            .build(Some(window_handle));

        let not_current_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attrs)
                .expect("failed to create GL context")
        };

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window_handle,
            NonZeroU32::new(self.width).unwrap(),
            NonZeroU32::new(self.height).unwrap(),
        );

        let surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &attrs)
                .expect("failed to create surface")
        };

        let context = not_current_context
            .make_current(&surface)
            .expect("failed to make context current");

        // swap interval to determine
        let _ = surface.set_swap_interval(
            &context,
            glutin::surface::SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
        );

        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|name| gl_display.get_proc_address(name))
        };

        let gl_state = GlState::new(&gl);

        let atlas = {
            let atlas_data = FontAtlasData::default();
            StaticFontAtlas::load(&gl, atlas_data).expect("failed to load font atlas")
        };

        // pixel ratio to determine
        let mut grid = TerminalGrid::new(
            &gl,
            atlas.into(),
            (self.width as i32, self.height as i32),
            1.0,
            &GlslVersion::Gl330,
        )
        .map_err(|e| Error::BeamTermError(e))?;

        grid.set_bg_alpha(&gl, self.bg_alpha);

        state.set_frame_callback(&event_queue.handle())?;
        state.set_grid_dims(grid.cell_size().width, grid.cell_size().height);

        Ok(Window {
            wayland_event_queue: event_queue,
            wayland_state: state,
            grid,
            gl,
            gl_state,
            gl_context: context,
            gl_surface: surface,
        })
    }
}

impl Window {
    pub fn update(&mut self) -> Result<(), Error> {
        self.wayland_event_queue
            .blocking_dispatch(&mut self.wayland_state)
            .unwrap();
        if self.wayland_state.is_redraw() {
            unsafe {
                use glow::HasContext;
                self.gl
                    .viewport(0, 0, self.width() as i32, self.height() as i32);
                self.gl.clear_color(0.1, 0.1, 0.1, 0.5);
                self.gl.clear(glow::COLOR_BUFFER_BIT);
            }

            // self.grid.render(&self.gl, &mut self.gl_state).unwrap();

            self.gl_surface.swap_buffers(&self.gl_context).unwrap();

            self.wayland_state.set_redraw(false);
            self.wayland_state
                .set_frame_callback(&self.wayland_event_queue.handle())
                .unwrap();
        }
        Ok(())
    }

    pub fn is_redraw(&self) -> bool {
        self.wayland_state.is_redraw()
    }

    pub fn update_states(&mut self) -> Result<(), Error> {
        self.wayland_event_queue
            .blocking_dispatch(&mut self.wayland_state)
            .map_err(|e| Error::WaylandDispatchError(e))?;
        Ok(())
    }

    pub fn reset_draw(&mut self) -> Result<(), Error> {
        self.wayland_state.set_redraw(false);
        self.wayland_state
            .set_frame_callback(&self.wayland_event_queue.handle())
            .unwrap();
        Ok(())
    }

    pub fn clear_screen(&self) {
        unsafe {
            use glow::HasContext;
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    pub fn events(&self) -> WindowEventQueue {
        self.wayland_state.events()
    }

    pub fn width(&self) -> u32 {
        self.wayland_state.width()
    }

    pub fn height(&self) -> u32 {
        self.wayland_state.height()
    }

    pub fn cols(&self) -> u16 {
        self.grid.terminal_size().cols
    }

    pub fn rows(&self) -> u16 {
        self.grid.terminal_size().rows
    }

    pub fn cell_height(&self) -> i32 {
        self.grid.cell_size().height
    }

    pub fn cell_width(&self) -> i32 {
        self.grid.cell_size().width
    }

    // Untested, pixel ratio to determine
    pub fn resize(&mut self, width: u32, height: u32) {
        let layer_surface = self.wayland_state.layer_surface().unwrap();
        layer_surface.set_size(width, height);
        self.wayland_event_queue
            .blocking_dispatch(&mut self.wayland_state)
            .unwrap();
        self.grid.resize(&self.gl, (width as i32, height as i32), 1.0).unwrap();
        self.wayland_state.set_grid_dims(width as i32, height as i32);
    }
}
