use crate::error::Error;
use crate::events::{
    AxisCode, ButtonCode, KeyCode, KeyEvent, MouseEvent, WindowEvent, WindowEventQueue,
};
use glutin::{context::NotCurrentGlContext, display::GlDisplay, prelude::GlSurface};
use wayland_client::{
    Connection, Dispatch, Proxy, WEnum, delegate_noop,
    protocol::{
        wl_callback, wl_compositor, wl_keyboard, wl_pointer, wl_registry, wl_seat, wl_surface,
    },
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, Anchor, KeyboardInteractivity, ZwlrLayerSurfaceV1},
};
use xkbcommon::xkb;

const PIXEL_RATIO: f32 = 1.0;

#[derive(Debug, Clone)]
pub struct WindowBuilder {
    pub namespace: String,
    pub width: u32,
    pub height: u32,
    pub anchors: Option<Anchor>,
    pub margin: Option<(i32, i32, i32, i32)>,
    pub exclusive_zone: Option<i32>,
    pub keyboard_interactivity: Option<KeyboardInteractivity>,
    pub layer: Layer,
    pub exclusive_edge: Option<Anchor>,
    pub font_name: Option<&'static str>,
    pub font_size: f32,
    pub bg_alpha: f32,
}

impl std::default::Default for WindowBuilder {
    fn default() -> Self {
        Self {
            namespace: "raclettui-layer-shell".to_string(),
            width: 100,
            height: 100,
            anchors: None,
            margin: None,
            exclusive_edge: None,
            keyboard_interactivity: None,
            layer: Layer::Overlay,
            exclusive_zone: None,
            font_name: None,
            font_size: 16.0,
            bg_alpha: 1.0,
        }
    }
}

pub(crate) struct WaylandState {
    // surface creation objects
    pub(crate) layer: zwlr_layer_shell_v1::Layer,
    pub(crate) namespace: String,
    pub(crate) anchors: Option<zwlr_layer_surface_v1::Anchor>,
    pub(crate) exclusive_zone: Option<i32>,
    pub(crate) margins: Option<(i32, i32, i32, i32)>,
    pub(crate) exclusive_edge: Option<zwlr_layer_surface_v1::Anchor>,
    pub(crate) keyboard_interactivity: Option<zwlr_layer_surface_v1::KeyboardInteractivity>,

    pub(crate) compositor: Option<wl_compositor::WlCompositor>,
    pub(crate) layer_shell: Option<ZwlrLayerShellV1>,

    pub(crate) surface: Option<wl_surface::WlSurface>,
    pub(crate) layer_surface: Option<ZwlrLayerSurfaceV1>,

    pub(crate) surface_configured: bool,
    pub(crate) frame_callback: Option<wl_callback::WlCallback>,
    pub(crate) needs_redraw: bool,

    pub(crate) window_width: u32,
    pub(crate) window_height: u32,

    // keyboard and mouse events related objects
    pub(crate) seat: Option<wl_seat::WlSeat>,
    pub(crate) keyboard: Option<wl_keyboard::WlKeyboard>,
    pub(crate) pointer: Option<wl_pointer::WlPointer>,

    pub(crate) events: WindowEventQueue,
    pub(crate) keymap: Option<xkb::Keymap>,
    pub(crate) keymap_state: Option<xkb::State>,

    pub(crate) grid_dims: Option<(i32, i32)>,
}

pub struct LayerShellWindow {
    pub(crate) gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    pub(crate) gl_context: glutin::context::PossiblyCurrentContext,
    pub(crate) gl_state: beamterm_core::GlState,
    pub(crate) gl: glow::Context,
    pub(crate) grid: beamterm_core::TerminalGrid,
    pub(crate) wayland_state: WaylandState,
    pub(crate) wayland_event_queue: wayland_client::EventQueue<WaylandState>,
}

impl WindowBuilder {
    pub fn new(layer: Layer) -> Self {
        Self {
            namespace: String::new(),
            width: 0,
            height: 0,
            anchors: None,
            margin: None,
            exclusive_zone: None,
            keyboard_interactivity: None,
            layer: layer,
            exclusive_edge: None,
            font_name: None,
            font_size: 0.,
            bg_alpha: 0.,
        }
    }

    fn init_wayland_state(&self) -> WaylandState {
        WaylandState {
            anchors: self.anchors.clone(),
            keyboard_interactivity: self.keyboard_interactivity.clone(),
            exclusive_zone: self.exclusive_zone.clone(),
            exclusive_edge: self.exclusive_edge.clone(),
            margins: self.margin.clone(),
            layer: self.layer.clone(),
            namespace: self.namespace.clone(),
            compositor: None,
            layer_shell: None,
            surface: None,
            layer_surface: None,
            surface_configured: false,
            frame_callback: None,
            needs_redraw: true,
            window_width: self.width,
            window_height: self.height,
            seat: None,
            keyboard: None,
            pointer: None,
            events: WindowEventQueue::new(),
            keymap: None,
            keymap_state: None,
            grid_dims: None,
        }
    }

    pub fn build(self) -> Result<LayerShellWindow, Error> {
        let wl_conn = Connection::connect_to_env().map_err(|e| Error::WaylandConnectError(e))?;
        let mut wl_event_queue = wl_conn.new_event_queue();
        let wl_qh = wl_event_queue.handle();

        let display = wl_conn.display();
        display.get_registry(&wl_qh, ());

        let mut wl_state = self.init_wayland_state();

        wl_event_queue
            .roundtrip(&mut wl_state)
            .map_err(|e| Error::WaylandDispatchError(e))?;
        wl_event_queue
            .roundtrip(&mut wl_state)
            .map_err(|e| Error::WaylandDispatchError(e))?;

        if !wl_state.surface_configured {
            return Err(Error::WaylandSurfaceConfigurationError);
        }

        let backend = wl_conn.backend();
        let wl_display_ptr = backend.display_ptr() as *mut std::ffi::c_void;
        if wl_display_ptr.is_null() {
            return Err(Error::WaylandDisplayPtrNull);
        }
        let display_ptr = unsafe { core::ptr::NonNull::new_unchecked(wl_display_ptr) };

        let raw_display_handle = raw_window_handle::RawDisplayHandle::Wayland(
            raw_window_handle::WaylandDisplayHandle::new(display_ptr),
        );
        let gl_display = unsafe {
            glutin::display::Display::new(
                raw_display_handle,
                glutin::display::DisplayApiPreference::Egl,
            )
            .map_err(|e| Error::GlutinError(e))?
        };

        let template = glutin::config::ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            // TODO: test with 0
            .with_depth_size(24)
            .build();

        let gl_config = unsafe {
            gl_display
                .find_configs(template)
                .map_err(|e| Error::GlutinError(e))?
                .next()
                .ok_or(Error::GlutinDisplayNull)?
        };

        let surface_ptr = wl_state
            .surface
            .as_ref()
            .ok_or(Error::WaylandSurfaceConfigurationError)?
            .id()
            .as_ptr() as *mut std::ffi::c_void;
        if surface_ptr.is_null() {
            return Err(Error::WaylandSurfacePtrNull);
        }
        let nonnull_surface_ptr = unsafe { core::ptr::NonNull::new_unchecked(surface_ptr) };
        // let gl_surface_raw = glutin::surface::RawSurface::Egl(surface_ptr);

        let window_handle = raw_window_handle::RawWindowHandle::Wayland(
            raw_window_handle::WaylandWindowHandle::new(nonnull_surface_ptr),
        );

        let context_attrs = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::OpenGl(Some(
                glutin::context::Version::new(3, 3),
            )))
            .build(Some(window_handle));

        let not_current_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attrs)
                .map_err(|e| Error::GlutinError(e))?
        };

        let attrs =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    window_handle,
                    std::num::NonZeroU32::new(wl_state.window_width)
                        .ok_or(Error::ZeroWindowWidth)?,
                    std::num::NonZeroU32::new(wl_state.window_height)
                        .ok_or(Error::ZeroWindowHeight)?,
                );

        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &attrs)
                .map_err(|e| Error::GlutinError(e))?
        };

        let gl_context = not_current_context
            .make_current(&gl_surface)
            .map_err(|e| Error::GlutinError(e))?;

        // TODO: swap interval to determine
        gl_surface
            .set_swap_interval(
                &gl_context,
                glutin::surface::SwapInterval::Wait(
                    std::num::NonZeroU32::new(1).ok_or(Error::ZeroSwapInterval)?,
                ),
            )
            .map_err(|e| Error::GlutinError(e))?;

        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|name| gl_display.get_proc_address(name))
        };
        let gl_state = beamterm_core::GlState::new(&gl);

        let font_name = match self.font_name {
            Some(font_name) => font_name,
            None => &self.get_default_font(),
        };

        let effective_font_size = self.font_size * PIXEL_RATIO;
        let rasterizer =
            beamterm_core::NativeGlyphRasterizer::new(&[font_name], effective_font_size)
                .map_err(|e| Error::BeamTermError(e))?;
        let atlas =
            beamterm_core::gl::DynamicFontAtlas::new(&gl, rasterizer, self.font_size, PIXEL_RATIO)
                .map_err(|e| Error::BeamTermError(e))?;

        // TODO: pixel ratio to determine
        let mut grid = beamterm_core::TerminalGrid::new(
            &gl,
            atlas.into(),
            (wl_state.window_width as i32, wl_state.window_height as i32),
            PIXEL_RATIO,
            &beamterm_core::GlslVersion::Gl330,
        )
        .map_err(|e| Error::BeamTermError(e))?;

        grid.set_bg_alpha(&gl, self.bg_alpha);

        wl_state.grid_dims = Some((grid.cell_size().width, grid.cell_size().height));

        let mut layer_shell_window = LayerShellWindow {
            gl_surface,
            gl_context,
            gl_state,
            gl,
            grid,
            wayland_state: wl_state,
            wayland_event_queue: wl_event_queue,
        };
        layer_shell_window.set_frame_callback()?;

        Ok(layer_shell_window)
    }

    #[inline]
    fn get_default_font(&self) -> String {
        let config = fontconfig::Fontconfig::new().unwrap();
        let font = config.find("monospace", None).unwrap();
        font.name
    }
}

impl LayerShellWindow {
    #[inline]
    fn set_frame_callback(&mut self) -> Result<(), Error> {
        if let Some(surface) = &self.wayland_state.surface {
            self.wayland_state.frame_callback =
                Some(surface.frame(&self.wayland_event_queue.handle(), ()));
            Ok(())
        } else {
            Err(Error::WaylandFrameCallbackError)
        }
    }

    #[inline]
    pub(crate) fn set_redraw(&mut self) -> Result<(), Error> {
        self.wayland_state.needs_redraw = false;
        self.set_frame_callback()
    }

    // Call this function on a window resize event
    pub fn resize_grid(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.grid
            .resize(&self.gl, (width as i32, height as i32), PIXEL_RATIO)
            .map_err(|e| Error::BeamTermError(e))?;
        self.wayland_state.grid_dims =
            Some((self.grid.cell_size().width, self.grid.cell_size().height));
        Ok(())
    }

    pub fn events(&self) -> WindowEventQueue {
        self.wayland_state.events.clone()
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(
                        name,
                        version,
                        qhandle,
                        (),
                    );
                    let surface = compositor.create_surface(qhandle, ());

                    state.compositor = Some(compositor.clone());
                    state.surface = Some(surface);
                }
                "zwlr_layer_shell_v1" => {
                    let layer_shell =
                        registry.bind::<ZwlrLayerShellV1, _, _>(name, version, qhandle, ());

                    let surface = match &state.surface {
                        Some(surface) => surface,
                        None => panic!(
                            "src/wgpu/wayland.rs unable to create zwlr_layer_shell_v1 surface, no base surface"
                        ),
                    };

                    // configuring layer shell surface
                    let layer_surface = layer_shell.get_layer_surface(
                        surface,
                        None,
                        state.layer,
                        state.namespace.clone(),
                        qhandle,
                        (),
                    );
                    if let Some(anchor) = state.anchors {
                        layer_surface.set_anchor(anchor);
                    }
                    layer_surface.set_size(state.window_width, state.window_height);
                    if let Some(zone) = state.exclusive_zone {
                        layer_surface.set_exclusive_zone(zone);
                    }
                    if let Some((top, right, bottom, left)) = state.margins {
                        layer_surface.set_margin(top, right, bottom, left);
                    }
                    if let Some(keyboard_interactivity) = state.keyboard_interactivity {
                        layer_surface.set_keyboard_interactivity(keyboard_interactivity);
                    }
                    if let Some(edge) = state.exclusive_edge {
                        layer_surface.set_exclusive_edge(edge);
                    }

                    state.layer_shell = Some(layer_shell);
                    state.layer_surface = Some(layer_surface);
                    // initial commit without buffer attached
                    surface.commit();
                }
                "wl_seat" => {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, version, qhandle, ());
                    state.seat = Some(seat);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        zwlr_layer_surface_v1: &ZwlrLayerSurfaceV1,
        event: <ZwlrLayerSurfaceV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        {
            if !state.surface_configured {
                zwlr_layer_surface_v1.ack_configure(serial);
                state.window_width = width;
                state.window_height = height;
                state.surface_configured = true;
            } else {
                zwlr_layer_surface_v1.ack_configure(serial);
                state.window_width = width;
                state.window_height = height;
                let event = WindowEvent::Resize { width, height };
                state.events.push(event);
            }
        }
    }
}

impl Dispatch<wl_callback::WlCallback, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _cb: &wl_callback::WlCallback,
        event: <wl_callback::WlCallback as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            state.needs_redraw = true;
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for WaylandState {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: <wl_seat::WlSeat as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities { capabilities } = event {
            if let WEnum::Value(capability) = capabilities {
                if capability.contains(wl_seat::Capability::Keyboard) {
                    let keyboard = seat.get_keyboard(qh, ());
                    state.keyboard = Some(keyboard);
                }
                if capability.contains(wl_seat::Capability::Pointer) {
                    let pointer = seat.get_pointer(qh, ());
                    state.pointer = Some(pointer);
                }
            }
        }
    }
}

// configures keymap and handles keyboard events
impl Dispatch<wl_keyboard::WlKeyboard, ()> for WaylandState {
    fn event(
        wl_state: &mut Self,
        _proxy: &wl_keyboard::WlKeyboard,
        event: <wl_keyboard::WlKeyboard as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Key { key, state, .. } => {
                if let WEnum::Value(key_state) = state {
                    if let Some(keymap_state) = &wl_state.keymap_state {
                        let window_event = new_keyboard_event(keymap_state, key, key_state as u32);
                        wl_state.events.push(window_event);
                    } else {
                        eprintln!("keymap state not configured, cannot get get keyboard event")
                    }
                }
            }
            wl_keyboard::Event::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                ..
            } => {
                wl_state.keymap_state.as_mut().unwrap().update_mask(
                    mods_depressed,
                    mods_latched,
                    mods_locked,
                    0,
                    0,
                    group,
                );
            }
            wl_keyboard::Event::Keymap { fd, size, .. } => {
                let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS as xkb::ContextFlags);
                let keymap_result = unsafe {
                    xkb::Keymap::new_from_fd(
                        &context,
                        fd,
                        size as usize,
                        xkb::KEYMAP_FORMAT_TEXT_V1,
                        xkb::KEYMAP_COMPILE_NO_FLAGS,
                    )
                };
                if let Ok(keymap) = keymap_result {
                    wl_state.keymap = keymap;
                } else if let Err(e) = keymap_result {
                    eprintln!("wl_keyboard event error getting keymap: {}", e);
                }
                if let Some(keymap) = &wl_state.keymap {
                    let keymap_state = xkb::State::new(keymap);
                    wl_state.keymap_state = Some(keymap_state);
                } else {
                    eprintln!("wl_keyboard event no keymap");
                }
            }
            _ => {}
        }
    }
}

#[inline]
fn new_keyboard_event(keymap_state: &xkb::State, key: u32, value: u32) -> WindowEvent {
    let keysym = keymap_state.key_get_one_sym(xkb::Keycode::new(key + 8));
    let shift = keymap_state.mod_name_is_active(xkb::MOD_NAME_SHIFT, xkb::STATE_MODS_EFFECTIVE);
    let ctrl = keymap_state.mod_name_is_active(xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE);
    let alt = keymap_state.mod_name_is_active(xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE);

    let keycode = match keysym {
        xkb::Keysym::BackSpace => KeyCode::Backspace,
        xkb::Keysym::Return => KeyCode::Enter,
        xkb::Keysym::Left => KeyCode::Left,
        xkb::Keysym::Right => KeyCode::Right,
        xkb::Keysym::Up => KeyCode::Up,
        xkb::Keysym::Down => KeyCode::Down,
        xkb::Keysym::Tab => KeyCode::Tab,
        xkb::Keysym::Delete => KeyCode::Delete,
        xkb::Keysym::Home => KeyCode::Home,
        xkb::Keysym::End => KeyCode::End,
        xkb::Keysym::Page_Up => KeyCode::PageUp,
        xkb::Keysym::Page_Down => KeyCode::PageDown,
        xkb::Keysym::Escape => KeyCode::Esc,
        xkb::Keysym::F1 => KeyCode::F(1),
        xkb::Keysym::F2 => KeyCode::F(2),
        xkb::Keysym::F3 => KeyCode::F(3),
        xkb::Keysym::F4 => KeyCode::F(4),
        xkb::Keysym::F5 => KeyCode::F(5),
        xkb::Keysym::F6 => KeyCode::F(6),
        xkb::Keysym::F7 => KeyCode::F(7),
        xkb::Keysym::F8 => KeyCode::F(8),
        xkb::Keysym::F9 => KeyCode::F(9),
        xkb::Keysym::F10 => KeyCode::F(10),
        xkb::Keysym::F11 => KeyCode::F(11),
        xkb::Keysym::F12 => KeyCode::F(12),
        _ => {
            let utf32 = xkb::keysym_to_utf32(keysym);
            if utf32 != 0 {
                let ch = std::char::from_u32(utf32).unwrap();
                KeyCode::Char(ch)
            } else {
                KeyCode::Unidentified
            }
        }
    };
    WindowEvent::Keyboard(KeyEvent {
        code: keycode,
        value,
        shift,
        alt,
        ctrl,
    })
}

impl Dispatch<wl_pointer::WlPointer, ()> for WaylandState {
    fn event(
        wl_state: &mut Self,
        _proxy: &wl_pointer::WlPointer,
        event: <wl_pointer::WlPointer as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Enter {
                surface_x,
                surface_y,
                ..
            } => {
                if let Some((cell_width, cell_height)) = wl_state.grid_dims {
                    let row = (surface_y / cell_height as f64).floor() as u32;
                    let col = (surface_x / cell_width as f64).floor() as u32;
                    let event = WindowEvent::Pointer(MouseEvent::Enter { row, col });
                    wl_state.events.push(event);
                }
            }
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                if let Some((cell_width, cell_height)) = wl_state.grid_dims {
                    let row = (surface_y / cell_height as f64).floor() as u32;
                    let col = (surface_x / cell_width as f64).floor() as u32;
                    let event = WindowEvent::Pointer(MouseEvent::Motion { row, col });
                    wl_state.events.push(event);
                }
            }
            wl_pointer::Event::Leave { .. } => {
                let event = WindowEvent::Pointer(MouseEvent::Leave);
                wl_state.events.push(event);
            }
            wl_pointer::Event::Axis { axis, value, .. } => {
                if let WEnum::Value(axis_value) = axis {
                    match axis_value {
                        wl_pointer::Axis::VerticalScroll => {
                            let event = WindowEvent::Pointer(MouseEvent::Axis {
                                code: AxisCode::VerticalScroll,
                                value,
                            });
                            wl_state.events.push(event);
                        }
                        wl_pointer::Axis::HorizontalScroll => {
                            let event = WindowEvent::Pointer(MouseEvent::Axis {
                                code: AxisCode::HorizontalScroll,
                                value,
                            });
                            wl_state.events.push(event);
                        }
                        _ => {}
                    }
                }
            }
            wl_pointer::Event::Button { button, state, .. } => {
                if let WEnum::Value(value) = state {
                    let event = new_pointer_button_event(button, value as u32);
                    wl_state.events.push(event);
                }
            }
            _ => {}
        }
    }
}

#[inline]
fn new_pointer_button_event(code: u32, value: u32) -> WindowEvent {
    let button_code = match code {
        272 => ButtonCode::Left,
        273 => ButtonCode::Right,
        274 => ButtonCode::Middle,
        _ => ButtonCode::Unknown,
    };
    WindowEvent::Pointer(MouseEvent::Button {
        code: button_code,
        value,
    })
}

delegate_noop!(WaylandState : ignore wl_compositor::WlCompositor);
delegate_noop!(WaylandState : ignore wl_surface::WlSurface);
delegate_noop!(WaylandState : ignore ZwlrLayerShellV1);
