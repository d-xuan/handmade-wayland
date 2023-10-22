use std::{cmp::max, f32::consts::PI};

pub struct Game {
    x_offset: u8,
    y_offset: u8,
    pitch_offset: i32,
    sample_index: f32, // TODO: Sync with period issues
}

pub struct PixelBuffer<'a> {
    pub data: &'a mut [u8],
    pub height: i32,
    pub width: i32,
    pub stride: i32,
}

pub struct SoundBuffer {
    pub data: Vec<u8>,
    pub bytes_per_sample: usize,
    pub sample_rate: u32,
    pub num_channels: u8,
}
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

impl Game {
    pub fn new() -> Game {
        Game {
            x_offset: 0,
            y_offset: 0,
            pitch_offset: 0,
            sample_index: 0.0,
        }
    }

    pub fn update_and_render(self: &mut Self, pixel_buffer: &mut PixelBuffer, keystate: &KeyState) {
        // Update offset on each timestep
        if keystate.left {
            self.x_offset = self.x_offset.wrapping_sub(25);
            if self.pitch_offset > -250 {
                self.pitch_offset -= 1;
            }
        } else if keystate.right {
            self.x_offset = self.x_offset.wrapping_add(25);
            if self.pitch_offset < 250 {
                self.pitch_offset += 1;
            }
        }

        if keystate.up {
            self.y_offset = self.y_offset.wrapping_sub(25);
            if self.pitch_offset < 250 {
                self.pitch_offset += 1;
            }
        } else if keystate.down {
            self.y_offset = self.y_offset.wrapping_add(25);
            if self.pitch_offset > -250 {
                self.pitch_offset -= 1;
            }
        }

        self.render(pixel_buffer);
    }

    fn render(self: &mut Self, pixel_buffer: &mut PixelBuffer) {
        let height = pixel_buffer.height;
        let width = pixel_buffer.width;
        let stride = pixel_buffer.stride;
        let bytes_per_pixel = pixel_buffer.stride / pixel_buffer.width;

        for y in 0..height {
            for x in 0..width {
                let pixel = (y * stride + x * bytes_per_pixel) as usize;
                pixel_buffer.data[pixel] = self // B
                    .x_offset
                    .wrapping_add(x as u8);
                pixel_buffer.data[pixel + 1] = self.y_offset.wrapping_add(y as u8);
                pixel_buffer.data[pixel + 2] = 0x00; // R
            }
        }
    }

    pub fn play_sound(self: &mut Self, sound_buffer: &mut SoundBuffer) {
        let tone_hz = 500.0 + self.pitch_offset as f32;
        let amplitude = 0.7;
        let length = sound_buffer.data.len();
        let channels = sound_buffer.num_channels;
        let sample_rate_f = sound_buffer.sample_rate as f32;

        // y = sin(kt)
        let k = 2.0 * tone_hz * PI;
        let mut t = self.sample_index / sample_rate_f;
        let mut y = amplitude * f32::sin(k * t);

        let mut i = 0;
        while i < length {
            let y_bytes = y.to_ne_bytes();
            for _channel in 0..channels {
                for b in y_bytes {
                    sound_buffer.data[i] = b;
                    i += 1;
                }
            }
            self.sample_index += 1.0;
            t = self.sample_index / sample_rate_f;
            y = amplitude * f32::sin(k * t);
        }
    }
}
