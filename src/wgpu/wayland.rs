use wayland_client::{
    Connection, Dispatch, QueueHandle, WEnum, delegate_noop,
    protocol::{
        wl_callback, wl_compositor, wl_keyboard, wl_pointer, wl_registry, wl_seat, wl_surface,
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
pub struct WgpuWaylandState {
    // surface creation objects
    layer: zwlr_layer_shell_v1::Layer,
    namespace: String,
    anchors: Vec<zwlr_layer_surface_v1::Anchor>,
    exclusive_zone: Option<i32>,
    margins: Option<(i32, i32, i32, i32)>,
    exclusive_edge: Option<zwlr_layer_surface_v1::Anchor>,
    keyboard_interactivity: Option<zwlr_layer_surface_v1::KeyboardInteractivity>,

    compositor: Option<wl_compositor::WlCompositor>,
    layer_shell: Option<ZwlrLayerShellV1>,

    pub surface: Option<wl_surface::WlSurface>,
    layer_surface: Option<ZwlrLayerSurfaceV1>,

    surface_configured: bool,
    pub frame_callback: Option<wl_callback::WlCallback>,
    pub needs_redraw: bool,

    pub window_width: u32,
    pub window_height: u32,

    // keyboard and mouse events related objects
    seat: Option<wl_seat::WlSeat>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    pointer: Option<wl_pointer::WlPointer>,

    pub events: WindowEventQueue,
    keymap: Option<xkb::Keymap>,
    keymap_state: Option<xkb::State>,
}

impl WgpuWaylandState {
    pub fn new(builder: &WindowBuilder) -> Self {
        Self {
            anchors: builder.anchors.clone(),
            keyboard_interactivity: builder.keyboard_interactivity.clone(),
            exclusive_zone: builder.exclusive_zone.clone(),
            exclusive_edge: builder.exclusive_edge.clone(),
            margins: builder.margin.clone(),
            layer: builder.layer.clone(),
            namespace: builder.namespace.clone(),
            compositor: None,
            layer_shell: None,
            surface: None,
            layer_surface: None,
            surface_configured: false,
            frame_callback: None,
            needs_redraw: true,
            window_width: builder.width,
            window_height: builder.height,
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

    pub fn set_frame_callback(&mut self, qh: &QueueHandle<WgpuWaylandState >) {
        if let Some(surface) = &self.surface {
            self.frame_callback = Some(surface.frame(&qh, ()));
        } else {
            panic!("src/wgpu/wayland.rs Unable to set frame callback, surface None");
        }
    }
}

// registry handler
impl Dispatch<wl_registry::WlRegistry, ()> for WgpuWaylandState {
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
                "zwlr_layer_shell_v1" => {
                    let layer_shell =
                        registry.bind::<ZwlrLayerShellV1, _, _>(name, version, qh, ());

                    let surface = match &state.surface {
                        Some(surface) => surface,
                        None => panic!("src/wgpu/wayland.rs unable to create zwlr_layer_shell_v1 surface, no base surface")
                    };

                    // configuring layer shell surface
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

                    state.layer_shell = Some(layer_shell);
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
impl Dispatch<ZwlrLayerSurfaceV1, ()> for WgpuWaylandState {
    fn event(
        state: &mut Self,
        layer_surface: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
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
        }
    }
}

// frame_callback for redraw when compositor ready
impl Dispatch<wl_callback::WlCallback, ()> for WgpuWaylandState {
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
impl Dispatch<wl_seat::WlSeat, ()> for WgpuWaylandState {
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
impl Dispatch<wl_keyboard::WlKeyboard, ()> for WgpuWaylandState {
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
                    if let Some(keymap_state) = &own_state.keymap_state {
                        let window_event = WindowEvent::new_keyboard_event(
                            keymap_state,
                            key,
                            key_state as u32,
                        );
                        own_state.events.push(window_event);
                    } else {
                        eprintln!("src/wgpu/wayland.rs keymap state not configured, cannot get get keyboard event")
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
                    eprintln!("src/wgpu/wayland.rs wl_keyboard event error getting keymap: {}", e);
                }
                if let Some(keymap) = &own_state.keymap {
                    let keymap_state = xkb::State::new(keymap);
                    own_state.keymap_state = Some(keymap_state);
                } else {
                    eprintln!("src/wgpu/wayland.rs wl_keyboard event no keymap");
                }
            }
            _ => {}
        }
    }
}

// handles pointer events
impl Dispatch<wl_pointer::WlPointer, ()> for WgpuWaylandState {
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
delegate_noop!(WgpuWaylandState : ignore wl_compositor::WlCompositor);
delegate_noop!(WgpuWaylandState : ignore wl_surface::WlSurface);
delegate_noop!(WgpuWaylandState : ignore ZwlrLayerShellV1);
