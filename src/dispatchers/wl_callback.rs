use libc::INT_MAX;
use wayland_client::{protocol::wl_callback, Connection, Dispatch, QueueHandle};

use crate::draw_frame;
use crate::State;

impl Dispatch<wl_callback::WlCallback, ()> for State {
    fn event(
        state: &mut Self,
        _callback: &wl_callback::WlCallback,
        event: wl_callback::Event,
        _udata: &(),
        _conn: &Connection,
        qh: &QueueHandle<State>,
    ) {
        if let wl_callback::Event::Done {
            callback_data: time,
        } = event
        {
            eprintln!("received wl_callback: {}", time);
            draw_frame(state, &qh);
            state.offset = state.offset.wrapping_add(1);

            state.surface.as_ref().unwrap().frame(&qh, ());
            state.surface.as_ref().unwrap().commit();
        }
    }
}
