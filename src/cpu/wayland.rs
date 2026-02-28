use std::{fs::File, os::unix::io::AsFd};

use tempfile::tempfile;
use wayland_client::{
    Connection, Dispatch, QueueHandle, WEnum, delegate_noop,
    protocol::{
        wl_buffer, wl_callback, wl_compositor, wl_keyboard, wl_pointer, wl_registry, wl_seat,
        wl_shm, wl_shm_pool, wl_surface,
    },
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
};
use xkbcommon::xkb;

use crate::{
    builder::WindowBuilder,
    events::{self, WindowEvent, WindowEventQueue},
};

// holds wayland states
pub struct CpuWaylandState {
    layer: zwlr_layer_shell_v1::Layer,
    namespace: String,
    anchors: Vec<zwlr_layer_surface_v1::Anchor>,
    exclusive_zone: Option<i32>,
    margins: Option<(i32, i32, i32, i32)>,
    exclusive_edge: Option<zwlr_layer_surface_v1::Anchor>,
    keyboard_interactivity: Option<zwlr_layer_surface_v1::KeyboardInteractivity>,

    compositor: Option<wl_compositor::WlCompositor>,
    shm: Option<wl_shm::WlShm>,
    layer_shell: Option<ZwlrLayerShellV1>,

    pub surface: Option<wl_surface::WlSurface>,
    layer_surface: Option<ZwlrLayerSurfaceV1>,

    pool: Option<wl_shm_pool::WlShmPool>,
    pub buffer: Option<wl_buffer::WlBuffer>,
    pub file: Option<File>,

    surface_configured: bool,
    pub frame_callback: Option<wl_callback::WlCallback>,
    pub needs_redraw: bool,

    pub window_width: u32,
    pub window_height: u32,

    seat: Option<wl_seat::WlSeat>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    pointer: Option<wl_pointer::WlPointer>,

    pub events: WindowEventQueue,
    keymap: Option<xkb::Keymap>,
    keymap_state: Option<xkb::State>,
}

impl CpuWaylandState {
    pub fn new(builder: &WindowBuilder) -> Self {
        let window_width = builder.width;
        let window_height = builder.height;

        Self {
            anchors: builder.anchors.clone(),
            keyboard_interactivity: builder.keyboard_interactivity.clone(),
            exclusive_zone: builder.exclusive_zone.clone(),
            exclusive_edge: builder.exclusive_edge.clone(),
            margins: builder.margin.clone(),
            layer: builder.layer.clone(),
            namespace: builder.namespace.clone(),
            compositor: None,
            shm: None,
            layer_shell: None,
            surface: None,
            layer_surface: None,
            pool: None,
            buffer: None,
            file: None,
            surface_configured: false,
            frame_callback: None,
            needs_redraw: true,
            window_width,
            window_height,
            seat: None,
            keyboard: None,
            pointer: None,
            events: WindowEventQueue::new(),
            keymap: None,
            keymap_state: None,
        }
    }

    pub fn is_surface_configured(&self) -> bool {
        self.surface_configured
    }

    pub fn is_redraw(&self) -> bool {
        self.needs_redraw
    }

    pub fn set_frame_callback(&mut self, qh: &QueueHandle<CpuWaylandState>) {
        if let Some(surface) = &self.surface {
            self.frame_callback = Some(surface.frame(&qh, ()));
        } else {
            panic!("src/cpu/wayland.rs Unable to set frame callback, surface None");
        }
    }
}

// registry handler
impl Dispatch<wl_registry::WlRegistry, ()> for CpuWaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, version, qh, ());
                    state.compositor = Some(compositor.clone());

                    let surface = compositor.create_surface(qh, ());
                    state.surface = Some(surface);
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, version, qh, ());
                    state.shm = Some(shm.clone());

                }
                "zwlr_layer_shell_v1" => {
                    let layer_shell =
                        registry.bind::<ZwlrLayerShellV1, _, _>(name, version, qh, ());
                    state.layer_shell = Some(layer_shell.clone());

                    let surface = state.surface.as_ref().unwrap();

                    let layer_surface = layer_shell.get_layer_surface(
                        surface,
                        None,
                        state.layer,
                        state.namespace.clone(),
                        qh,
                        (),
                    );
                    for anchor in &state.anchors {
                        layer_surface.set_anchor(*anchor);
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

                    state.layer_surface = Some(layer_surface);
                    // initial commit without buffer attached
                    surface.commit();
                }
                "wl_seat" => {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, version, qh, ());
                    state.seat = Some(seat);
                }
                _ => {}
            }
        }
    }
}

// acknowledge configure event
impl Dispatch<ZwlrLayerSurfaceV1, ()> for CpuWaylandState {
    fn event(
        state: &mut Self,
        layer_surface: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        {
            layer_surface.ack_configure(serial);
            state.window_width = width;
            state.window_height = height;
            state.surface_configured = true;
            state.events.push(WindowEvent::new_resize_event(width, height));

            let file = tempfile().expect("src/cpu/wayland.rs zwlr_layer_surface_v1 event unable to create file");
            let size = (state.window_width * state.window_height * 4) as i32;

            file.set_len(size as u64).expect("src/cpu/wayland.rs zwlr_layer_surface_v1 event unable to set file len");

            if let Some(shm) = &state.shm {
                let pool = shm.create_pool(file.as_fd(), size, qh, ());

                let buffer = pool.create_buffer(
                    0,
                    state.window_width as i32,
                    state.window_height as i32,
                    (state.window_width * 4) as i32,
                    wl_shm::Format::Argb8888,
                    qh,
                    (),
                );
                state.pool = Some(pool);
                state.buffer = Some(buffer);
            }
            state.file = Some(file);
        }
    }
}

// frame_callback for redraw when compositor ready
impl Dispatch<wl_callback::WlCallback, ()> for CpuWaylandState {
    fn event(
        state: &mut Self,
        _: &wl_callback::WlCallback,
        event: wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            state.needs_redraw = true;
        }
    }
}

// Gets keyboard and pointer proxies
// No touch support yet
impl Dispatch<wl_seat::WlSeat, ()> for CpuWaylandState {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: <wl_seat::WlSeat as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
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
impl Dispatch<wl_keyboard::WlKeyboard, ()> for CpuWaylandState {
    fn event(
        own_state: &mut Self,
        _proxy: &wl_keyboard::WlKeyboard,
        event: <wl_keyboard::WlKeyboard as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Key { key, state, .. } => {
                if let WEnum::Value(key_state) = state {
                    let window_event = WindowEvent::new_keyboard_event(
                        own_state.keymap_state.as_ref().unwrap(),
                        key,
                        key_state as u32,
                    );
                    own_state.events.push(window_event);
                }
            }
            wl_keyboard::Event::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                ..
            } => {
                own_state.keymap_state.as_mut().unwrap().update_mask(
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
                    own_state.keymap = keymap;
                } else if let Err(e) = keymap_result {
                    eprintln!("src/cpu/wayland.rs wl_keyboard event error getting keymap: {}", e);
                }
                if let Some(keymap) = &own_state.keymap {
                    let keymap_state = xkb::State::new(keymap);
                    own_state.keymap_state = Some(keymap_state);
                } else {
                    eprintln!("src/cpu/wayland.rs wl_keyboard event no keymap");
                }
            }
            _ => {}
        }
    }
}

// handles pointer events
impl Dispatch<wl_pointer::WlPointer, ()> for CpuWaylandState {
    fn event(
        own_state: &mut Self,
        _proxy: &wl_pointer::WlPointer,
        event: <wl_pointer::WlPointer as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Enter {
                surface_x,
                surface_y,
                ..
            } => {
                let event = WindowEvent::new_pointer_enter_event(surface_x, surface_y);
                own_state.events.push(event);
            }
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                let event = WindowEvent::new_pointer_motion_event(surface_x, surface_y);
                own_state.events.push(event);
            }
            wl_pointer::Event::Leave { .. } => {
                let event = WindowEvent::new_pointer_leave_event();
                own_state.events.push(event);
            }
            wl_pointer::Event::Axis { axis, value, .. } => {
                if let WEnum::Value(axis_value) = axis {
                    match axis_value {
                        wl_pointer::Axis::VerticalScroll => {
                            let event = WindowEvent::new_pointer_axis_event(
                                events::AxisCode::VerticalScroll,
                                value,
                            );
                            own_state.events.push(event);
                        }
                        wl_pointer::Axis::HorizontalScroll => {
                            let event = WindowEvent::new_pointer_axis_event(
                                events::AxisCode::HorizontalScroll,
                                value,
                            );
                            own_state.events.push(event);
                        }
                        _ => {}
                    }
                }
            }
            wl_pointer::Event::Button { button, state, .. } => {
                if let WEnum::Value(value) = state {
                    let event = WindowEvent::new_pointer_button_event(button, value as u32);
                    own_state.events.push(event);
                }
            }
            _ => {}
        }
    }
}

// ignore unused
delegate_noop!(CpuWaylandState: ignore wl_compositor::WlCompositor);
delegate_noop!(CpuWaylandState: ignore wl_surface::WlSurface);
delegate_noop!(CpuWaylandState: ignore wl_shm::WlShm);
delegate_noop!(CpuWaylandState: ignore wl_shm_pool::WlShmPool);
delegate_noop!(CpuWaylandState: ignore wl_buffer::WlBuffer);
delegate_noop!(CpuWaylandState: ignore ZwlrLayerShellV1);
