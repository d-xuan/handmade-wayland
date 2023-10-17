use wayland_client::{Connection, Dispatch, QueueHandle};

use wayland_protocols::xdg::shell::client::xdg_toplevel;

use crate::State;

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for State {
    fn event(
        state: &mut Self,
        _pointer: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
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
