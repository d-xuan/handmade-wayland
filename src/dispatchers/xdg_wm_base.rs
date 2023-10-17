use wayland_client::{Connection, Dispatch, QueueHandle};

use wayland_protocols::xdg::shell::client::xdg_wm_base;

use crate::State;

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
