use std::time::Duration;

use wayland_client::{protocol::wl_callback, Connection, Dispatch, QueueHandle};

use crate::frame_draw;
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
            frame_draw(state, &qh);
            state.offset = state.offset.wrapping_add(1);
            state.surface.as_ref().unwrap().frame(&qh, time);
            state.surface.as_ref().unwrap().commit();
        }
    }
}

impl Dispatch<wl_callback::WlCallback, u32> for State {
    fn event(
        state: &mut Self,
        _callback: &wl_callback::WlCallback,
        event: wl_callback::Event,
        prevtime: &u32,
        _conn: &Connection,
        qh: &QueueHandle<State>,
    ) {
        if let wl_callback::Event::Done {
            callback_data: time,
        } = event
        {
            let delta_t: u64 = (time - *prevtime).into();
            println!(
                "Current fps: {}",
                1.0 / Duration::from_millis(delta_t).as_secs_f64()
            );
            frame_draw(state, &qh);
            state.offset = state.offset.wrapping_add(1);
            state.surface.as_ref().unwrap().frame(&qh, time);
            state.surface.as_ref().unwrap().commit();
        }
    }
}
