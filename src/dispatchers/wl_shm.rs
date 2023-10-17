use wayland_client::{
    protocol::wl_shm::{self},
    Connection, Dispatch, QueueHandle,
};

use crate::State;
impl Dispatch<wl_shm::WlShm, ()> for State {
    fn event(
        _state: &mut Self,
        _shm: &wl_shm::WlShm,
        _event: wl_shm::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        // WlShm events provide information about supported events.
        // We'll only deal with argb8888 for now.
    }
}
