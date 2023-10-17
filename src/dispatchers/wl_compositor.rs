use wayland_client::{protocol::wl_compositor, Connection, Dispatch, QueueHandle};

use crate::State;
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
