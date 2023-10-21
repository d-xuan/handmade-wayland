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
