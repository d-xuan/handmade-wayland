use std::{fs::File, os::fd::AsFd};

use wayland_client::Connection;
use wayland_client::{
    protocol::{
        wl_compositor, wl_seat,
        wl_shm::{self},
        wl_shm_pool, wl_surface,
    },
    QueueHandle,
};
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
    data: Option<memmap::MmapMut>,
    width: i32,
    height: i32,
    bytes_per_pixel: i32,
    pool: Option<wl_shm_pool::WlShmPool>,
    offset: u8,
    running: bool,
}

const BYTES_PER_PIXEL: i32 = 4;
impl State {
    fn new(width: i32, height: i32) -> State {
        State {
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
            offset: 0,
            bytes_per_pixel: BYTES_PER_PIXEL,
            running: true,
        }
    }
}

pub fn run(width: i32, height: i32) {
    // Initialise program state
    let mut state = State::new(width, height);

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

    // Main loop
    while state.running {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

fn draw_frame(state: &mut State, qh: &QueueHandle<State>) {
    let height = state.height;
    let width = state.width;
    let offset = state.offset;
    let bytes_per_pixel = state.bytes_per_pixel;
    let data = state.data.as_mut().unwrap();
    let stride = width * bytes_per_pixel;

    // Draw checkboxed background
    // TODO: Create ARGB structs to better encapsulate this
    // Look into the byteorder crate
    // state.offset += 1;
    for y in 0..height as usize {
        for x in 0..width as usize {
            let pixel = y * stride as usize + x * bytes_per_pixel as usize;
            data[pixel] = offset.wrapping_add(x as u8);
            data[pixel + 1] = offset.wrapping_add(y as u8);
            data[pixel + 2] = 0x00;
        }
    }

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
