use crate::State;
use wayland_client::{protocol::wl_pointer, Connection, Dispatch, QueueHandle};

impl Dispatch<wl_pointer::WlPointer, ()> for State {
    fn event(
        _state: &mut Self,
        _pointer: &wl_pointer::WlPointer,
        _event: wl_pointer::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        // no events needed yet
    }
}
