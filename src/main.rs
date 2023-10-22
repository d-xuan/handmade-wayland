use handmade_hero::{self, KeyState, PixelBuffer};
mod pulseaudio;
mod shm;

use epoll;
use pulseaudio::pulse_init;
use std::{
    cell::RefCell,
    fs::File,
    os::fd::{AsFd, AsRawFd, OwnedFd},
    rc::Rc,
};
use wayland_client::{
    protocol::{
        __interfaces, wl_buffer, wl_callback, wl_compositor, wl_keyboard, wl_pointer, wl_registry,
        wl_seat, wl_shm, wl_shm_pool, wl_surface,
    },
    Connection, Dispatch, EventQueue, QueueHandle, WEnum,
};
use wayland_protocols::{
    xdg,
    xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base},
};
use xkbcommon::{
    xkb,
    xkb::ffi::{
        xkb_keymap_new_from_string, XKB_CONTEXT_NO_FLAGS, XKB_KEYMAP_COMPILE_NO_FLAGS,
        XKB_KEYMAP_FORMAT_TEXT_V1,
    },
};

impl Dispatch<wl_buffer::WlBuffer, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        buffer: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandState>,
    ) {
        if let wl_buffer::Event::Release = event {
            buffer.destroy();
        }
    }
}

impl Dispatch<wl_callback::WlCallback, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _callback: &wl_callback::WlCallback,
        event: wl_callback::Event,
        _udata: &(),
        _conn: &Connection,
        qh: &QueueHandle<WaylandState>,
    ) {
        if let wl_callback::Event::Done {
            callback_data: time,
        } = event
        {
            wl_frame_draw(state, &qh);
            state.surface.as_ref().unwrap().frame(&qh, time);
            state.surface.as_ref().unwrap().commit();
        }
    }
}

impl Dispatch<wl_callback::WlCallback, u32> for WaylandState {
    fn event(
        state: &mut Self,
        _callback: &wl_callback::WlCallback,
        event: wl_callback::Event,
        prevtime: &u32,
        _conn: &Connection,
        qh: &QueueHandle<WaylandState>,
    ) {
        if let wl_callback::Event::Done {
            callback_data: time,
        } = event
        {
            let delta_t: u64 = (time - *prevtime).into();
            eprint!("ms per frame: {}\r", delta_t);
            wl_frame_draw(state, &qh);
            state.surface.as_ref().unwrap().frame(&qh, time);
            state.surface.as_ref().unwrap().commit();
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _compositor: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandState>,
    ) {
        // wl_compositor::Event has no associated events
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _pointer: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandState>,
    ) {
        match event {
            wl_keyboard::Event::Keymap {
                format,
                fd,
                size: _,
            } => {
                if let WEnum::Value(wl_keyboard::KeymapFormat::XkbV1) = format {
                    xkb_configure(state, fd);
                }
            }
            wl_keyboard::Event::Modifiers {
                serial: _,
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
            } => {
                state.xkb_state.as_mut().unwrap().update_mask(
                    mods_depressed,
                    mods_latched,
                    mods_locked,
                    group,
                    group,
                    group,
                );
            }

            wl_keyboard::Event::Key {
                serial: _,
                time: _,
                key,
                state: key_state,
            } => {
                let key_sym_name = xkb_keysym_get(state.xkb_state.as_ref().unwrap(), key);
                dbg!(&key_sym_name);
                match key_state {
                    WEnum::Value(wl_keyboard::KeyState::Pressed) => match key_sym_name.as_str() {
                        "q" => state.running = false,
                        "w" => state.keystate.up = true,
                        "a" => state.keystate.left = true,
                        "s" => state.keystate.down = true,
                        "d" => state.keystate.right = true,
                        &_ => {}
                    },
                    WEnum::Value(wl_keyboard::KeyState::Released) => match key_sym_name.as_str() {
                        "w" => state.keystate.up = false,
                        "a" => state.keystate.left = false,
                        "s" => state.keystate.down = false,
                        "d" => state.keystate.right = false,
                        &_ => {}
                    },
                    _ => {} // close match key_state
                }
            }
            _ => {} // close match event
        }
    }
}

fn xkb_configure(state: &mut WaylandState, fd: OwnedFd) {
    let xkb_context = xkb::Context::new(XKB_CONTEXT_NO_FLAGS);

    let file = File::from(fd);
    let xkb_keymap = unsafe {
        let map_shm = memmap::MmapOptions::new().map(&file).unwrap();
        let s = map_shm.as_ptr();
        let ptr = xkb_keymap_new_from_string(
            xkb_context.get_raw_ptr(),
            s as *const i8,
            XKB_KEYMAP_FORMAT_TEXT_V1,
            XKB_KEYMAP_COMPILE_NO_FLAGS,
        );

        if ptr.is_null() {
            None
        } else {
            Some(xkb::Keymap::from_raw_ptr(ptr))
        }
    }
    .expect("Keymap compilatio nto succeed");

    let xkb_state = xkb::State::new(&xkb_keymap);

    state.xkb_context = Some(xkb_context);
    state.xkb_state = Some(xkb_state);
    state.xkb_keymap = Some(xkb_keymap);
}

fn xkb_keysym_get(xkb_state: &xkb::State, keycode: u32) -> String {
    let xkb_keycode = keycode + 8;
    xkb_state.key_get_utf8(xkb_keycode.into())
}

impl Dispatch<wl_pointer::WlPointer, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _pointer: &wl_pointer::WlPointer,
        _event: wl_pointer::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandState>,
    ) {
        // no events needed yet
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        udata: &(),
        _conn: &Connection,
        qh: &QueueHandle<WaylandState>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            eprintln!("Registry advertised interface {}", interface);
            if interface == __interfaces::WL_SHM_INTERFACE.name {
                eprintln!("Binding shm registry");
                state.shm = Some(registry.bind(name, version, qh, *udata));
            } else if interface == __interfaces::WL_SEAT_INTERFACE.name {
                eprintln!("Binding wl_seat interface");
                state.seat = Some(registry.bind(name, version, qh, *udata))
            } else if interface == __interfaces::WL_COMPOSITOR_INTERFACE.name {
                eprintln!("Issueing compositor bind request");
                state.compositor = Some(registry.bind(name, version, qh, *udata))
            } else if interface == xdg::shell::client::__interfaces::XDG_WM_BASE_INTERFACE.name {
                state.xdg_wm_base = Some(registry.bind(name, version, qh, *udata));
            }
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _udata: &(),
        _conn: &Connection,
        qh: &QueueHandle<WaylandState>,
    ) {
        if let wl_seat::Event::Capabilities { capabilities } = event {
            if let WEnum::Value(capability) = capabilities {
                // TODO: Store and release when done?
                if capability.intersects(wl_seat::Capability::Keyboard) {
                    seat.get_keyboard(qh, ());
                }

                if capability.intersects(wl_seat::Capability::Pointer) {
                    seat.get_pointer(qh, ());
                }
            }
        }
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _pointer: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandState>,
    ) {
        //wl_shm_pool::Event has no associated events
    }
}

impl Dispatch<wl_shm::WlShm, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _shm: &wl_shm::WlShm,
        _event: wl_shm::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandState>,
    ) {
        // WlShm events provide information about supported events.
        // We'll only deal with xrgb8888 for now.
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _shm: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandState>,
    ) {
        // No events needed yet
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for WaylandState {
    fn event(
        state: &mut Self,
        surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _udata: &(),
        _conn: &Connection,
        qh: &QueueHandle<WaylandState>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            eprintln!("received xdg_surface_configure");
            surface.ack_configure(serial);

            wl_frame_draw(state, &qh);
            state.surface.as_ref().unwrap().commit();
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _pointer: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandState>,
    ) {
        match event {
            xdg_toplevel::Event::Close => {
                state.running = false;
            }
            xdg_toplevel::Event::Configure {
                width,
                height,
                states: _,
            } => {
                if width != 0 && height != 0 {
                    // width = height = 0 means we get to decide the size
                    state.width = width;
                    state.height = height;
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        xdg_wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<WaylandState>,
    ) {
        // Server asked if we're still alive. Reply back
        if let xdg_wm_base::Event::Ping { serial } = event {
            xdg_wm_base.pong(serial);
        }
    }
}

struct WaylandState {
    // Wayland
    shm: Option<wl_shm::WlShm>,
    seat: Option<wl_seat::WlSeat>,
    compositor: Option<wl_compositor::WlCompositor>,
    xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,
    surface: Option<wl_surface::WlSurface>,

    // Xdg
    xdg_surface: Option<xdg_surface::XdgSurface>,
    xdg_toplevel: Option<xdg_toplevel::XdgToplevel>,

    // Backbuffer
    data: Option<memmap::MmapMut>,
    width: i32,
    height: i32,
    bytes_per_pixel: i32,
    pool: Option<wl_shm_pool::WlShmPool>,

    // XKB
    xkb_state: Option<xkb::State>,
    xkb_context: Option<xkb::Context>,
    xkb_keymap: Option<xkb::Keymap>,
    keystate: KeyState,
    running: bool,

    // Application
    game: Rc<RefCell<handmade_hero::Game>>,
}

impl WaylandState {
    fn new(width: i32, height: i32) -> WaylandState {
        WaylandState {
            shm: None,
            seat: None,
            compositor: None,
            xdg_wm_base: None,
            surface: None,
            xdg_surface: None,
            xdg_toplevel: None,
            pool: None,
            data: None,
            height,
            width,
            game: Rc::new(RefCell::new(handmade_hero::Game::new())),
            xkb_state: None,
            xkb_context: None,
            xkb_keymap: None,
            bytes_per_pixel: BYTES_PER_PIXEL,
            keystate: KeyState::new(),
            running: true,
        }
    }
}

const RESOLUTION_WIDTH: i32 = 1920;
const RESOLUTION_HEIGHT: i32 = 1080;
const BYTES_PER_PIXEL: i32 = 4;
const SAMPLE_RATE: u32 = 48000;
const NUM_CHANNELS: u8 = 2;

fn main() {
    // Setup wayland event queue
    let (mut state, mut event_queue) = wl_init(RESOLUTION_WIDTH, RESOLUTION_HEIGHT);
    let wayland_fd = event_queue.as_fd();
    let wayland_event = epoll::Event {
        events: libc::EPOLLIN as u32,
        data: wayland_fd.as_raw_fd() as u64,
    };
    let epoll_fd = epoll::create(true).unwrap();
    epoll::ctl(
        epoll_fd,
        epoll::ControlOptions::EPOLL_CTL_ADD,
        wayland_fd.as_raw_fd(),
        wayland_event,
    )
    .unwrap();

    let pulse_mainloop = pulse_init(&state.game, SAMPLE_RATE, NUM_CHANNELS);

    // Main loop
    while state.running {
        // Flush outgoing wayland events
        event_queue.flush().unwrap();
        event_queue.dispatch_pending(&mut state).unwrap();

        // Synchronise read from event queue.
        let read_guard = event_queue.prepare_read().unwrap();

        // Check sockets to see if ready
        let mut events = vec![epoll::Event { data: 0, events: 0 }];
        let wayland_socket_ready = epoll::wait(epoll_fd, 0, &mut events).unwrap() != 0;

        if wayland_socket_ready {
            read_guard.read().unwrap();
            event_queue.dispatch_pending(&mut state).unwrap();
        } else {
            pulse_mainloop.borrow_mut().iterate(false);
        }
    }
}

fn wl_init(width: i32, height: i32) -> (WaylandState, EventQueue<WaylandState>) {
    // Initialise program state
    let mut state = WaylandState::new(width, height);

    // Create a Wayland connection object from the connection.
    let conn = Connection::connect_to_env().expect("initial connection to wayland should succeed");

    // Retrive the WlDisplay wayland object from the connection.
    let display = conn.display();

    // Create an event queue for event processing
    // All responses from the server will come through this queue.
    let mut event_queue = conn.new_event_queue();

    // And get its handle to associate new object to it.
    let qh = event_queue.handle();

    // Create a wl_registry object by sending a wl_display_get_registry request
    display.get_registry(&qh, ());
    event_queue
        .roundtrip(&mut state)
        .expect("initial roundtrip should succeed");

    // Ask the compositor for a surface
    state.surface = Some(state.compositor.as_ref().unwrap().create_surface(&qh, ()));

    // Convert the surface into an xdg_surface for desktop applications
    state.xdg_surface = Some(state.xdg_wm_base.as_ref().unwrap().get_xdg_surface(
        state.surface.as_ref().unwrap(),
        &qh,
        (),
    ));

    // Assign our xdg_surface the top-level role for a full-fledged desktop window
    state.xdg_toplevel = Some(state.xdg_surface.as_ref().unwrap().get_toplevel(&qh, ()));

    // Set XDG window title
    state
        .xdg_toplevel
        .as_ref()
        .unwrap()
        .set_title("Handmade Hero".to_string());

    // draw frame
    let stride: i32 = width * BYTES_PER_PIXEL;
    let size = height * stride;
    let fd = shm::allocate_shm_file(size).expect("should be able to allocate shared memory");
    let file = File::from(fd);

    state.data = Some(unsafe { memmap::MmapOptions::new().map_mut(&file).unwrap() });

    state.pool = Some(state.shm.as_ref().unwrap().create_pool(
        file.as_fd(),
        size.try_into().unwrap(),
        &qh,
        (),
    ));

    // draw_frame(&mut state, &qh);
    state.surface.as_ref().unwrap().commit();
    state.surface.as_ref().unwrap().frame(&qh, ());

    return (state, event_queue);
}

fn wl_frame_draw(state: &mut WaylandState, qh: &QueueHandle<WaylandState>) {
    let height = state.height;
    let width = state.width;
    let bytes_per_pixel = state.bytes_per_pixel;
    let data = state.data.as_mut().unwrap();
    let stride = width * bytes_per_pixel;

    // Draw checkboxed background
    let mut pixel_buffer = PixelBuffer {
        data,
        height,
        width,
        stride,
    };

    state
        .game
        .borrow_mut()
        .update_and_render(&mut pixel_buffer, &state.keystate);

    let buffer = Some(state.pool.as_ref().unwrap().create_buffer(
        0,
        width,
        height,
        stride,
        wl_shm::Format::Xrgb8888,
        &qh,
        (),
    ));

    state
        .surface
        .as_ref()
        .unwrap()
        .attach(buffer.as_ref(), 0, 0);

    state
        .surface
        .as_ref()
        .unwrap()
        .damage_buffer(0, 0, state.width, state.height);
}
