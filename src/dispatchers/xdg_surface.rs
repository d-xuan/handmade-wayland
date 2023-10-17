use memmap;
use std::{fs::File, os::fd::AsFd};
use wayland_client::{
    protocol::{
        wl_buffer::WlBuffer,
        wl_shm::{self},
    },
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::xdg::shell::client::xdg_surface;

use crate::shm;
use crate::State;

impl Dispatch<xdg_surface::XdgSurface, ()> for State {
    fn event(
        state: &mut Self,
        surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _udata: &(),
        _conn: &Connection,
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
