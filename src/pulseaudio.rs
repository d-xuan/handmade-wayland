use std::{cell::RefCell, mem::size_of, ops::Deref, rc::Rc};

use handmade_hero::{Game, SoundBuffer};
use pulse::{
    context::{self, Context, FlagSet},
    mainloop::standard::Mainloop,
    proplist::Proplist,
    sample::{Format, Spec},
    stream::{SeekMode, Stream},
};

pub fn pulse_init(
    game: &Rc<RefCell<Game>>,
    sample_rate: u32,
    num_channels: u8,
) -> Rc<RefCell<Mainloop>> {
    // Create a mainloop API and connection to the default server
    let mut proplist = Proplist::new().unwrap();
    proplist
        .set_str(
            pulse::proplist::properties::APPLICATION_NAME,
            "HandmadeHero",
        )
        .unwrap();
    let mainloop = Rc::new(RefCell::new(
        Mainloop::new().expect("Failed to create mainloop"),
    ));
    let context = Rc::new(RefCell::new(
        Context::new_with_proplist(mainloop.borrow().deref(), "HandmadeHero", &proplist)
            .expect("Failed to create new context"),
    ));
    context
        .borrow_mut()
        .connect(None, FlagSet::NOFLAGS, None)
        .expect("Failed to connect to context");

    // This function defines a callback so the server will tell us it's state.
    // Our callback will wait for the state to be ready.  The callback will
    // modify the variable so we know when we have a connection and it's
    // ready.
    // pa_context_set_state_callback(pa_ctx, pa_state_cb, &pa_ready);
    let context_ref = Rc::clone(&context);
    let context_ready = Rc::new(RefCell::new(false));
    let context_ready_ref = Rc::clone(&context_ready);
    context
        .borrow_mut()
        .set_state_callback(Some(Box::new(move || {
            match context_ref.borrow().get_state() {
                context::State::Terminated | context::State::Failed => {
                    panic!("Context state failed");
                }
                context::State::Ready => {
                    *context_ready_ref.borrow_mut() = true;
                }
                _ => {}
            }
        })));
    while *context_ready.borrow() == false {
        mainloop.borrow_mut().iterate(true);
    }

    // Setup specification
    let spec = Spec {
        format: Format::FLOAT32NE,
        rate: sample_rate,
        channels: num_channels,
    };
    assert!(spec.is_valid());
    let stream = Rc::new(RefCell::new(
        Stream::new(&mut context.borrow_mut(), "GameAudio", &spec, None)
            .expect("Failed to create audio stream"),
    ));

    // Setup write callback
    let stream_ref = Rc::clone(&stream);
    let game_ref = Rc::clone(game);
    stream
        .borrow_mut()
        .set_write_callback(Some(Box::new(move |length: usize| {
            let data: Vec<u8> = vec![0; length];
            let mut sound_buffer = SoundBuffer {
                data,
                bytes_per_sample: size_of::<f32>(),
                sample_rate,
                num_channels,
            };

            game_ref.borrow_mut().play_sound(&mut sound_buffer);
            stream_ref
                .borrow_mut()
                .write(sound_buffer.data.as_slice(), None, 0, SeekMode::Relative)
                .unwrap();
        })));

    stream
        .borrow_mut()
        .connect_playback(
            None,
            None,
            pulse::stream::FlagSet::INTERPOLATE_TIMING | pulse::stream::FlagSet::AUTO_TIMING_UPDATE,
            None,
            None,
        )
        .expect("Stream playback connection to succeed");

    return mainloop;
}
