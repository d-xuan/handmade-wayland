use wayland_client::{protocol::wl_surface, Connection, Dispatch, QueueHandle};

use crate::State;

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
