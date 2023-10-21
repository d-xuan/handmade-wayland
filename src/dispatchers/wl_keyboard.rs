use std::{fs::File, os::fd::OwnedFd};

use crate::State;
use wayland_client::{protocol::wl_keyboard, Connection, Dispatch, QueueHandle, WEnum};

use xkbcommon::xkb::{
    self,
    ffi::{
        xkb_keymap_new_from_string, XKB_CONTEXT_NO_FLAGS, XKB_KEYMAP_COMPILE_NO_FLAGS,
        XKB_KEYMAP_FORMAT_TEXT_V1,
    },
};

pub struct KeyState {
    pub up: bool,
    pub left: bool,
    pub right: bool,
    pub down: bool,
}

impl KeyState {
    pub fn new() -> Self {
        KeyState {
            up: false,
            left: false,
            right: false,
            down: false,
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn event(
        state: &mut Self,
        _pointer: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        match event {
            wl_keyboard::Event::Keymap {
                format,
                fd,
                size: _,
            } => {
                if let WEnum::Value(wl_keyboard::KeymapFormat::XkbV1) = format {
                    xkb_configure(state, fd);
                }
            }
            wl_keyboard::Event::Modifiers {
                serial: _,
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
            } => {
                state.xkb_state.as_mut().unwrap().update_mask(
                    mods_depressed,
                    mods_latched,
                    mods_locked,
                    group,
                    group,
                    group,
                );
            }

            wl_keyboard::Event::Key {
                serial: _,
                time: _,
                key,
                state: key_state,
            } => {
                let key_sym_name = xkb_keysym_get(state.xkb_state.as_ref().unwrap(), key);
                match key_state {
                    WEnum::Value(wl_keyboard::KeyState::Pressed) => match key_sym_name.as_str() {
                        "q" => state.running = false,
                        "w" => state.keystate.up = true,
                        "a" => state.keystate.left = true,
                        "s" => state.keystate.down = true,
                        "d" => state.keystate.right = true,
                        &_ => {}
                    },
                    WEnum::Value(wl_keyboard::KeyState::Released) => match key_sym_name.as_str() {
                        "w" => state.keystate.up = false,
                        "a" => state.keystate.left = false,
                        "s" => state.keystate.down = false,
                        "d" => state.keystate.right = false,
                        &_ => {}
                    },
                    _ => {} // close match key_state
                }
            }
            _ => {} // close match event
        }
    }
}

fn xkb_configure(state: &mut State, fd: OwnedFd) {
    let xkb_context = xkb::Context::new(XKB_CONTEXT_NO_FLAGS);

    let file = File::from(fd);
    let xkb_keymap = unsafe {
        let map_shm = memmap::MmapOptions::new().map(&file).unwrap();
        let s = map_shm.as_ptr();
        let ptr = xkb_keymap_new_from_string(
            xkb_context.get_raw_ptr(),
            s as *const i8,
            XKB_KEYMAP_FORMAT_TEXT_V1,
            XKB_KEYMAP_COMPILE_NO_FLAGS,
        );

        if ptr.is_null() {
            None
        } else {
            Some(xkb::Keymap::from_raw_ptr(ptr))
        }
    }
    .expect("Keymap compilatio nto succeed");

    let xkb_state = xkb::State::new(&xkb_keymap);

    state.xkb_context = Some(xkb_context);
    state.xkb_state = Some(xkb_state);
    state.xkb_keymap = Some(xkb_keymap);
}

fn xkb_keysym_get(xkb_state: &xkb::State, keycode: u32) -> String {
    let xkb_keycode = keycode + 8;
    xkb_state.key_get_utf8(xkb_keycode.into())
}
