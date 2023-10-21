use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::xdg::shell::client::xdg_surface;

use crate::frame_draw;
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
            eprintln!("received xdg_surface_configure");
            surface.ack_configure(serial);

            frame_draw(state, &qh);
            state.surface.as_ref().unwrap().commit();
        }
    }
}
