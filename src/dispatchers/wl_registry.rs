use wayland_client::{
    protocol::{__interfaces, wl_registry},
    Connection, Dispatch, QueueHandle,
};

use wayland_protocols::xdg::{self};

use crate::State;

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        udata: &(),
        _conn: &Connection,
        qh: &QueueHandle<State>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            eprintln!("Registry advertised interface {}", interface);
            if interface == __interfaces::WL_SHM_INTERFACE.name {
                eprintln!("Binding shm registry");
                state.shm = Some(registry.bind(name, version, qh, *udata));
            } else if interface == __interfaces::WL_SEAT_INTERFACE.name {
                eprintln!("Binding wl_seat interface");
                state.seat = Some(registry.bind(name, version, qh, *udata))
            } else if interface == __interfaces::WL_COMPOSITOR_INTERFACE.name {
                eprintln!("Issueing compositor bind request");
                state.compositor = Some(registry.bind(name, version, qh, *udata))
            } else if interface == xdg::shell::client::__interfaces::XDG_WM_BASE_INTERFACE.name {
                state.xdg_wm_base = Some(registry.bind(name, version, qh, *udata));
            }
        }
    }
}
