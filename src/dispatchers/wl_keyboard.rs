use wayland_client::{protocol::wl_keyboard, Connection, Dispatch, QueueHandle, WEnum};

use crate::State;

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn event(
        app_state: &mut Self,
        _pointer: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        match event {
            wl_keyboard::Event::Key {
                serial: _,
                time: _,
                key,
                state,
            } => {
                if let WEnum::Value(wl_keyboard::KeyState::Pressed) = state {
                    if key == 16 {
                        // Q on qwerty
                        app_state.running = false;
                    }
                }
            }
            _ => {} // implement other keyboard inputs as needed
        }
    }
}
