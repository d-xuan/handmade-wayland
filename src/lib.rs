use wayland_client::protocol::{
    wl_compositor, wl_seat,
    wl_shm::{self},
    wl_surface,
};
use wayland_client::Connection;
use wayland_protocols::xdg::shell::client::xdg_surface;
use wayland_protocols::xdg::shell::client::xdg_toplevel;
use wayland_protocols::xdg::shell::client::xdg_wm_base;

mod dispatchers;
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

impl State {
    fn new() -> State {
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

pub fn run() {
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
