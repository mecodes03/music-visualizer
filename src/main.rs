use num::complex::Complex32;
use raylib::prelude::*;
use std::sync::Mutex;

mod fourier_transform;

// TODO: use array everywhere instead of vec since most lengts(kinda estimated) are known at compile time.

const MAX_VOLUME: f32 = 1.0;
const VOLUME_CHANGE_BY: f32 = 0.05;
const INITIAL_VOLUME: f32 = 0.5;
const SEEK_BY: f32 = 5.0;
const FPS: u32 = 20;

static FREQUENCY_COUNT: Mutex<usize> = Mutex::new(0);
static SAMPLE_COUNT: Mutex<usize> = Mutex::new(0);
static CAPACITY: usize = 1024;
/** samples in one Frame of 60 FPS*/
static CURRENT_SAMPLES: Mutex<[f32; CAPACITY]> = Mutex::new([0.0; CAPACITY]);
/** frequency of samples in one Frame of 60 FPS*/
static CURRENT_FRAME_FREQUENCIES: Mutex<[f32; CAPACITY]> = Mutex::new([0.0; CAPACITY]);

#[derive(Copy, Clone)]
enum DisplayType {
    Samples,
    Frequency,
}

static DISPLAY_TYPE: Mutex<DisplayType> = Mutex::new(DisplayType::Samples);
/** Programme using safe raylib rust bindings. */
fn main() {
    const WIDTH: i32 = 640;
    const HEIGHT: i32 = 480;

    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Musializer")
        .build();
    rl.set_target_fps(FPS);

    let raylib_audio_device = RaylibAudio::init_audio_device().expect("audio init failed");
    let music = raylib_audio_device
        .new_music("assets/audio.mp3")
        .expect("sound load failed");
    music.play_stream();

    let mut audio_stream_processor = |_samples: &mut [f32], _channels: u32| {
        let samples: Vec<Complex32> = Vec::from(&*_samples)
            .iter()
            .step_by(2)
            .map(|&n| Complex32::from(n))
            .collect();

        match *DISPLAY_TYPE.lock().unwrap() {
            DisplayType::Samples => {
                let mut len = _samples.len();
                if _samples.len() > CAPACITY {
                    len = CAPACITY
                }
                for f in 0..len {
                    let mut global_samples = CURRENT_SAMPLES.lock().unwrap();
                    global_samples[f] = _samples[f];
                }
                *SAMPLE_COUNT.lock().unwrap() = len;
            }

            DisplayType::Frequency => {
                let fft = fourier_transform::fft(&samples);
                let mut len = fft.len();
                if len > CAPACITY {
                    len = CAPACITY
                }
                for f in 0..len {
                    let mut global_frequencies = CURRENT_FRAME_FREQUENCIES.lock().unwrap();
                    global_frequencies[f] = fft[f];
                }
                *FREQUENCY_COUNT.lock().unwrap() = len;
            }
        }
    };

    let _guard = attach_audio_stream_processor_to_music(&music, &mut audio_stream_processor);

    let mut volume: f32 = INITIAL_VOLUME;

    while !&rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        handle_keyboard(&music, d.get_key_pressed(), &mut volume);

        match *DISPLAY_TYPE.lock().unwrap() {
            DisplayType::Frequency => {
                let count = 64;
                if count > 0 {
                    let frequencies = *CURRENT_FRAME_FREQUENCIES.lock().unwrap();

                    let cell_width = WIDTH as f32 / count as f32;
                    let cell_height = HEIGHT as f32 / 256.0 as f32;

                    for f in 0..count {
                        let bar_height = frequencies[f] * cell_height;
                        let rect = Rectangle::new(
                            f as f32 * cell_width,
                            HEIGHT as f32 / 2.0 - bar_height,
                            cell_width,
                            bar_height,
                        );
                        d.draw_rectangle_rec(rect, color::Color::RED);
                    }
                }
            }

            DisplayType::Samples => {
                let count = *SAMPLE_COUNT.lock().unwrap();
                if count > 0 {
                    let samples = *CURRENT_SAMPLES.lock().unwrap();
                    // println!("_samples: {:?}", samples);
                    let cell_width = WIDTH as f32 / count as f32;

                    for f in 0..count {
                        let bar_height = samples[f] * HEIGHT as f32 / 2.0;
                        let rect = if bar_height > 0.0 {
                            Rectangle::new(
                                f as f32 * cell_width,
                                HEIGHT as f32 / 2.0 - bar_height,
                                cell_width,
                                bar_height,
                            )
                        } else {
                            Rectangle::new(
                                f as f32 * cell_width,
                                HEIGHT as f32 / 2.0,
                                cell_width,
                                -bar_height,
                            )
                        };
                        d.draw_rectangle_rec(rect, color::Color::RED);
                    }
                }
            }
        }
        music.update_stream();
    }
}

fn handle_keyboard(music: &Music<'_>, current_key: Option<KeyboardKey>, volume: &mut f32) {
    match current_key {
        Some(key) => match key {
            KeyboardKey::KEY_H => {
                let current_time_played = music.get_time_played();
                let seek = if current_time_played <= SEEK_BY {
                    0.0
                } else {
                    current_time_played - SEEK_BY
                };
                music.seek_stream(seek);
            }

            KeyboardKey::KEY_L => {
                let current_time_played = music.get_time_played();
                let music_length = music.get_time_length();
                let seek = if current_time_played + SEEK_BY > music_length {
                    music_length
                } else {
                    current_time_played + SEEK_BY
                };
                music.seek_stream(seek);
            }

            KeyboardKey::KEY_SPACE => {
                if music.is_stream_playing() {
                    music.pause_stream();
                } else {
                    music.resume_stream();
                }
            }

            KeyboardKey::KEY_DOWN => {
                println!("volume down");
                *volume = if *volume < 0.1 { 0.0 } else { *volume - 0.1 };
                music.set_volume(*volume);
            }

            KeyboardKey::KEY_UP => {
                println!("volume up");
                *volume = if *volume < 0.9 {
                    *volume + VOLUME_CHANGE_BY
                } else {
                    MAX_VOLUME
                };
                music.set_volume(*volume);
            }

            KeyboardKey::KEY_S => *DISPLAY_TYPE.lock().unwrap() = DisplayType::Samples,
            KeyboardKey::KEY_F => *DISPLAY_TYPE.lock().unwrap() = DisplayType::Frequency,
            _ => {}
        },
        None => {}
    }
}
