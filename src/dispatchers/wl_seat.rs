use crate::State;
use wayland_client::{protocol::wl_seat, Connection, Dispatch, QueueHandle, WEnum};

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _udata: &(),
        _conn: &Connection,
        qh: &QueueHandle<State>,
    ) {
        if let wl_seat::Event::Capabilities { capabilities } = event {
            if let WEnum::Value(capability) = capabilities {
                // TODO: Store and release when done?
                if capability.intersects(wl_seat::Capability::Keyboard) {
                    seat.get_keyboard(qh, ());
                }

                if capability.intersects(wl_seat::Capability::Pointer) {
                    seat.get_pointer(qh, ());
                }
            }
        }
    }
}
