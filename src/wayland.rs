//! Wayland connection and layer-shell surface handling shared by both
//! rendering backends.
//!
//! The CPU backend additionally uses the bound `wl_shm` global to allocate
//! its shared-memory buffers; everything else (surface creation, configure
//! handling, frame callbacks and input) is identical for both backends.

use wayland_client::{
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum, delegate_noop,
    protocol::{
        wl_buffer, wl_callback, wl_compositor, wl_keyboard, wl_pointer, wl_registry, wl_seat,
        wl_shm, wl_shm_pool, wl_surface,
    },
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
};
use xkbcommon::xkb;

use crate::{
    builder::WindowBuilder,
    events::{AxisCode, WindowEvent, WindowEventQueue},
    Error,
};

/// Wayland objects and window state shared by the CPU and wgpu backends.
pub(crate) struct WaylandState {
    // globals
    compositor: Option<wl_compositor::WlCompositor>,
    layer_shell: Option<ZwlrLayerShellV1>,
    pub shm: Option<wl_shm::WlShm>,
    seat: Option<wl_seat::WlSeat>,

    // surface
    pub surface: Option<wl_surface::WlSurface>,
    layer_surface: Option<ZwlrLayerSurfaceV1>,

    /// Set once the compositor sent the first configure event.
    configured: bool,
    /// Set when a frame callback fired and the compositor is ready for a new frame.
    pub needs_redraw: bool,
    /// Set when the compositor closed the layer surface.
    pub closed: bool,

    /// Size requested through the [`WindowBuilder`].
    requested_width: u32,
    requested_height: u32,
    /// Size of the surface as configured by the compositor.
    pub width: u32,
    pub height: u32,

    // input
    keyboard: Option<wl_keyboard::WlKeyboard>,
    pointer: Option<wl_pointer::WlPointer>,
    keymap_state: Option<xkb::State>,
    pub events: WindowEventQueue,
}

impl WaylandState {
    fn new(builder: &WindowBuilder) -> Self {
        Self {
            compositor: None,
            layer_shell: None,
            shm: None,
            seat: None,
            surface: None,
            layer_surface: None,
            configured: false,
            needs_redraw: true,
            closed: false,
            requested_width: builder.width,
            requested_height: builder.height,
            width: builder.width,
            height: builder.height,
            keyboard: None,
            pointer: None,
            keymap_state: None,
            events: WindowEventQueue::new(),
        }
    }

    /// Requests a frame callback on the surface. Must be called before the
    /// commit it should be attached to.
    pub fn request_frame(&mut self, qh: &QueueHandle<Self>) -> Result<(), Error> {
        let surface = self.surface.as_ref().ok_or(Error::WaylandFrameCallbackError)?;
        surface.frame(qh, ());
        Ok(())
    }
}

/// Connects to the Wayland compositor, creates the layer surface described by
/// `builder` and waits for its first configure event.
pub(crate) fn connect(
    builder: &WindowBuilder,
) -> Result<(Connection, EventQueue<WaylandState>, WaylandState), Error> {
    validate(builder)?;

    let conn = Connection::connect_to_env().map_err(Error::WaylandConnectError)?;
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    conn.display().get_registry(&qh, ());

    let mut state = WaylandState::new(builder);

    // bind globals
    event_queue.roundtrip(&mut state).map_err(Error::WaylandDispatchError)?;

    let compositor = state
        .compositor
        .as_ref()
        .ok_or(Error::MissingGlobal("wl_compositor"))?;
    let layer_shell = state
        .layer_shell
        .as_ref()
        .ok_or(Error::MissingGlobal("zwlr_layer_shell_v1"))?;

    // create the layer surface, globals may be advertised in any order
    // so this can only happen once all of them are bound.
    let surface = compositor.create_surface(&qh, ());
    let layer_surface = layer_shell.get_layer_surface(
        &surface,
        None,
        builder.layer,
        builder.namespace.clone(),
        &qh,
        (),
    );
    configure_layer_surface(&layer_surface, builder);

    // initial commit without a buffer attached, the compositor answers with a configure event
    surface.commit();
    state.surface = Some(surface);
    state.layer_surface = Some(layer_surface);

    while !state.configured {
        if state.closed {
            return Err(Error::WindowClosed);
        }
        event_queue
            .blocking_dispatch(&mut state)
            .map_err(Error::WaylandDispatchError)?;
    }

    Ok((conn, event_queue, state))
}

/// Rejects configurations the compositor would answer with a protocol error.
fn validate(builder: &WindowBuilder) -> Result<(), Error> {
    use zwlr_layer_surface_v1::Anchor;
    let anchor = builder.anchor_flags();
    if builder.width == 0 && !anchor.contains(Anchor::Left | Anchor::Right) {
        return Err(Error::InvalidConfiguration(
            "width is 0 but the surface is not anchored to both the left and right edges",
        ));
    }
    if builder.height == 0 && !anchor.contains(Anchor::Top | Anchor::Bottom) {
        return Err(Error::InvalidConfiguration(
            "height is 0 but the surface is not anchored to both the top and bottom edges",
        ));
    }
    Ok(())
}

fn configure_layer_surface(layer_surface: &ZwlrLayerSurfaceV1, builder: &WindowBuilder) {
    // set_anchor replaces the previous anchor, so all anchors are sent at once
    layer_surface.set_anchor(builder.anchor_flags());
    layer_surface.set_size(builder.width, builder.height);
    if let Some(zone) = builder.exclusive_zone {
        layer_surface.set_exclusive_zone(zone);
    }
    if let Some((top, right, bottom, left)) = builder.margin {
        layer_surface.set_margin(top, right, bottom, left);
    }
    if let Some(keyboard_interactivity) = builder.keyboard_interactivity {
        if keyboard_interactivity == zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand
            && layer_surface.version() < 4
        {
            eprintln!("raclettui: compositor does not support on-demand keyboard interactivity, ignoring");
        } else {
            layer_surface.set_keyboard_interactivity(keyboard_interactivity);
        }
    }
    if let Some(edge) = builder.exclusive_edge {
        if layer_surface.version() >= 5 {
            layer_surface.set_exclusive_edge(edge);
        } else {
            eprintln!("raclettui: compositor does not support exclusive edges, ignoring");
        }
    }
}

/// Dispatches already queued events, then blocks until the compositor is
/// ready for a new frame.
///
/// Returns [`Error::WindowClosed`] if the compositor closed the surface.
pub(crate) fn wait_for_frame(
    event_queue: &mut EventQueue<WaylandState>,
    state: &mut WaylandState,
) -> Result<(), Error> {
    event_queue
        .dispatch_pending(state)
        .map_err(Error::WaylandDispatchError)?;
    while !state.needs_redraw && !state.closed {
        event_queue
            .blocking_dispatch(state)
            .map_err(Error::WaylandDispatchError)?;
    }
    if state.closed {
        return Err(Error::WindowClosed);
    }
    state.needs_redraw = false;
    Ok(())
}

/// Binds a global, clamping the version to the one this crate was built with.
fn bind<I, U>(
    registry: &wl_registry::WlRegistry,
    name: u32,
    version: u32,
    qh: &QueueHandle<WaylandState>,
    udata: U,
) -> I
where
    I: Proxy + 'static,
    U: Send + Sync + 'static,
    WaylandState: Dispatch<I, U>,
{
    registry.bind::<I, U, WaylandState>(name, version.min(I::interface().version), qh, udata)
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                "wl_compositor" => state.compositor = Some(bind(registry, name, version, qh, ())),
                "zwlr_layer_shell_v1" => state.layer_shell = Some(bind(registry, name, version, qh, ())),
                "wl_shm" => state.shm = Some(bind(registry, name, version, qh, ())),
                // only the first seat is used
                "wl_seat" if state.seat.is_none() => state.seat = Some(bind(registry, name, version, qh, ())),
                _ => {}
            }
        }
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        layer_surface: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure { serial, width, height } => {
                layer_surface.ack_configure(serial);
                // a zero size means the client is free to pick its own size
                state.width = if width == 0 { state.requested_width } else { width };
                state.height = if height == 0 { state.requested_height } else { height };
                state.configured = true;
            }
            zwlr_layer_surface_v1::Event::Closed => {
                state.closed = true;
                state.events.push(WindowEvent::Closed);
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_callback::WlCallback, ()> for WaylandState {
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
impl Dispatch<wl_seat::WlSeat, ()> for WaylandState {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities { capabilities: WEnum::Value(capability) } = event {
            // capabilities can be re-announced, only create each device once
            if capability.contains(wl_seat::Capability::Keyboard) && state.keyboard.is_none() {
                state.keyboard = Some(seat.get_keyboard(qh, ()));
            }
            if capability.contains(wl_seat::Capability::Pointer) && state.pointer.is_none() {
                state.pointer = Some(seat.get_pointer(qh, ()));
            }
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for WaylandState {
    fn event(
        own_state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Keymap { fd, size, .. } => {
                let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
                let keymap_result = unsafe {
                    xkb::Keymap::new_from_fd(
                        &context,
                        fd,
                        size as usize,
                        xkb::KEYMAP_FORMAT_TEXT_V1,
                        xkb::KEYMAP_COMPILE_NO_FLAGS,
                    )
                };
                match keymap_result {
                    Ok(Some(keymap)) => own_state.keymap_state = Some(xkb::State::new(&keymap)),
                    Ok(None) => eprintln!("raclettui: unable to compile keymap"),
                    Err(e) => eprintln!("raclettui: error reading keymap: {}", e),
                }
            }
            wl_keyboard::Event::Key { key, state: WEnum::Value(key_state), .. } => {
                if let Some(keymap_state) = &own_state.keymap_state {
                    let window_event =
                        WindowEvent::new_keyboard_event(keymap_state, key, key_state as u32);
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
                if let Some(keymap_state) = own_state.keymap_state.as_mut() {
                    keymap_state.update_mask(mods_depressed, mods_latched, mods_locked, 0, 0, group);
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for WaylandState {
    fn event(
        own_state: &mut Self,
        _: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let window_event = match event {
            wl_pointer::Event::Enter { surface_x, surface_y, .. } => {
                WindowEvent::new_pointer_enter_event(surface_x, surface_y)
            }
            wl_pointer::Event::Motion { surface_x, surface_y, .. } => {
                WindowEvent::new_pointer_motion_event(surface_x, surface_y)
            }
            wl_pointer::Event::Leave { .. } => WindowEvent::new_pointer_leave_event(),
            wl_pointer::Event::Axis { axis: WEnum::Value(axis), value, .. } => match axis {
                wl_pointer::Axis::VerticalScroll => {
                    WindowEvent::new_pointer_axis_event(AxisCode::VerticalScroll, value)
                }
                wl_pointer::Axis::HorizontalScroll => {
                    WindowEvent::new_pointer_axis_event(AxisCode::HorizontalScroll, value)
                }
                _ => return,
            },
            wl_pointer::Event::Button { button, state: WEnum::Value(value), .. } => {
                WindowEvent::new_pointer_button_event(button, value as u32)
            }
            _ => return,
        };
        own_state.events.push(window_event);
    }
}

// ignore unused
delegate_noop!(WaylandState: ignore wl_compositor::WlCompositor);
delegate_noop!(WaylandState: ignore wl_surface::WlSurface);
delegate_noop!(WaylandState: ignore wl_shm::WlShm);
delegate_noop!(WaylandState: ignore wl_shm_pool::WlShmPool);
delegate_noop!(WaylandState: ignore wl_buffer::WlBuffer);
delegate_noop!(WaylandState: ignore ZwlrLayerShellV1);
