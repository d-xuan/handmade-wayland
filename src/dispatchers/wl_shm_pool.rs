use wayland_client::{protocol::wl_shm_pool, Connection, Dispatch, QueueHandle};

use crate::State;

impl Dispatch<wl_shm_pool::WlShmPool, ()> for State {
    fn event(
        _state: &mut Self,
        _pointer: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        //wl_shm_pool::Event has no associated events
    }
}
