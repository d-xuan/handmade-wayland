use std::{fs::File, os::fd::AsFd};

use memmap;
use wayland_client::{
    protocol::{
        __interfaces,
        wl_buffer::{self, WlBuffer},
        wl_compositor, wl_keyboard, wl_registry, wl_seat,
        wl_shm::{self},
        wl_shm_pool, wl_surface,
    },
    Connection, Dispatch, QueueHandle, WEnum,
};
use wayland_protocols::xdg::shell::client::xdg_surface;
use wayland_protocols::xdg::shell::client::xdg_wm_base;
use wayland_protocols::xdg::{self, shell::client::xdg_toplevel};

mod shm;

struct State {
    shm: Option<wl_shm::WlShm>,
    seat: Option<wl_seat::WlSeat>,
    compositor: Option<wl_compositor::WlCompositor>,
    xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,
    surface: Option<wl_surface::WlSurface>,
    xdg_surface: Option<xdg_surface::XdgSurface>,
    xdg_toplevel: Option<xdg_toplevel::XdgToplevel>,
    running: bool,
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        udata: &(),
        _conn: &Connection,
        qh: &QueueHandle<State>,
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

impl Dispatch<wl_compositor::WlCompositor, ()> for State {
    fn event(
        _state: &mut Self,
        _compositor: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        // wl_compositor::Event has no associated events
    }
}

impl Dispatch<wl_shm::WlShm, ()> for State {
    fn event(
        _state: &mut Self,
        _shm: &wl_shm::WlShm,
        _event: wl_shm::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        // WlShm events provide information about supported events.
        // We'll only deal with argb8888 for now.
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for State {
    fn event(
        _state: &mut Self,
        _shm: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        // No events needed yet
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _udata: &(),
        _conn: &Connection,
        qh: &QueueHandle<State>,
    ) {
        if let wl_seat::Event::Capabilities { capabilities } = event {
            if let WEnum::Value(capability) = capabilities {
                if capability.intersects(wl_seat::Capability::Keyboard) {
                    seat.get_keyboard(qh, ());
                }
            }
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _state: &mut Self,
        xdg_wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        // Server asked if we're still alive. Reply back
        if let xdg_wm_base::Event::Ping { serial } = event {
            xdg_wm_base.pong(serial);
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn event(
        app_state: &mut Self,
        pointer: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        udata: &(),
        conn: &Connection,
        qh: &QueueHandle<State>,
    ) {
        match event {
            wl_keyboard::Event::Key {
                serial,
                time,
                key,
                state,
            } => {
                if let WEnum::Value(wl_keyboard::KeyState::Pressed) = state {
                    if key == 0x18 {
                        // Q on qwerty
                        app_state.running = false;
                    }
                }
            }
            _ => {} // implement other keyboard inputs as needed
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for State {
    fn event(
        state: &mut Self,
        surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        udata: &(),
        conn: &Connection,
        qh: &QueueHandle<State>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            surface.ack_configure(serial);

            let buffer = draw_frame(state, qh);
            let surface = state.surface.as_ref().unwrap();
            surface.attach(Some(&buffer), 0, 0);
            surface.commit();
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for State {
    fn event(
        state: &mut Self,
        pointer: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        udata: &(),
        conn: &Connection,
        qh: &QueueHandle<State>,
    ) {
        //TODO(wednesday): implement other events
        match event {
            xdg_toplevel::Event::Close => {
                state.running = false;
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for State {
    fn event(
        _state: &mut Self,
        _pointer: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        //wl_shm_pool::Event has no associated events
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for State {
    fn event(
        _state: &mut Self,
        buffer: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        if let wl_buffer::Event::Release = event {
            buffer.destroy();
        }
    }
}

impl State {
    fn new() -> Self {
        State {
            shm: None,
            seat: None,
            compositor: None,
            xdg_wm_base: None,
            surface: None,
            xdg_surface: None,
            xdg_toplevel: None,
            running: true,
        }
    }
}

fn main() {
    // Initialise program state
    let mut state = State::new();

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

    // Render any pending buffers to the surface
    state.surface.as_ref().unwrap().commit();

    // Main loop
    while state.running {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

fn draw_frame(state: &State, qh: &QueueHandle<State>) -> WlBuffer {
    // Allocate a 1920x1080x4 backbuffer shm pool
    // TODO: Clean this up a bit
    // TODO: How do we avoid drawing a frame on every configure?
    let width: i32 = 1920;
    let height: i32 = 1080;
    let stride: i32 = width * 4;
    let size = height * stride;
    let fd = shm::allocate_shm_file(size).expect("should be able to allocate shared memory");
    let file = File::from(fd);

    let mut data = unsafe { memmap::MmapOptions::new().map_mut(&file).unwrap() };

    let pool =
        state
            .shm
            .as_ref()
            .unwrap()
            .create_pool(file.as_fd(), size.try_into().unwrap(), &qh, ());

    let buffer = pool.create_buffer(0, width, height, stride, wl_shm::Format::Argb8888, &qh, ());

    pool.destroy();

    // Draw checkboxed background
    // TODO: Create ARGB structs to better encapsulate this
    for y in 0..height as usize {
        for x in 0..stride as usize {
            data[y * stride as usize + x] = 0xFF
        }
    }

    return buffer;
}
