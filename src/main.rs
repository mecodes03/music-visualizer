use num::complex::Complex32;
use raylib::prelude::*;
use std::sync::Mutex;

mod fourier_transform;

// TODO: use array everywhere instead of vec since most lengts(kinda estimated) are known at compile time.

const MAX_VOLUME: f32 = 1.0;
const MIN_VOLUME: f32 = 0.0;
const VOLUME_CHANGE_BY: f32 = 0.05;
const INITIAL_VOLUME: f32 = 0.5;
const SEEK_BY: f32 = 5.0;
const FPS: u32 = 60;

const MAX_FFT_LENGHT: usize = 512;
const MAX_SAMPLE_CAP: usize = 1024;

/** samples in one Frame of 60 FPS*/
static CURRENT_SAMPLES: Mutex<[f32; MAX_SAMPLE_CAP]> = Mutex::new([0.0; MAX_SAMPLE_CAP]);
static SAMPLE_COUNT: Mutex<usize> = Mutex::new(0);

/** frequency of samples in one Frame of 60 FPS*/
static CURRENT_FRAME_FREQUENCIES: Mutex<[f32; MAX_FFT_LENGHT]> = Mutex::new([0.0; MAX_FFT_LENGHT]);
static FREQUENCY_COUNT: Mutex<usize> = Mutex::new(0);

static MAX_AMP: Mutex<f32> = Mutex::new(0.0);

#[derive(Copy, Clone)]
enum DisplayType {
    Samples,
    Frequency,
}

static DISPLAY_TYPE: Mutex<DisplayType> = Mutex::new(DisplayType::Samples);
/** Programme using safe raylib rust bindings. */
fn main() {
    const WIDTH: i32 = 800;
    const HEIGHT: i32 = 600;

    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Musializer")
        .build();
    rl.set_target_fps(FPS);

    let raylib_audio_device = RaylibAudio::init_audio_device().expect("audio init failed");
    let music = raylib_audio_device
        .new_music("assets/song3.mp3")
        .expect("sound load failed");
    music.play_stream();

    let mut audio_stream_processor =
        |_samples: &mut [f32], _channels: u32| match *DISPLAY_TYPE.lock().unwrap() {
            DisplayType::Samples => {
                // only take left samples (mono basically)
                let mut len = _samples.len() / 2;
                if len > MAX_SAMPLE_CAP {
                    len = MAX_SAMPLE_CAP
                }
                let mut global_samples = CURRENT_SAMPLES.lock().unwrap();
                for f in 0..len {
                    global_samples[f] = _samples[2 * f];
                }

                *SAMPLE_COUNT.lock().unwrap() = len;
            }

            DisplayType::Frequency => {
                let mono: Vec<f32> = _samples.iter().step_by(2).copied().collect();
                // TODO: make MAX_FFT_LENGHT mutable so user can choose how many he wants (128, 64, 32, 16)
                let len = if mono.len() > MAX_FFT_LENGHT {
                    MAX_FFT_LENGHT
                } else {
                    mono.len().next_power_of_two()
                };

                let mut samples: Vec<Complex32> = mono.iter().map(|x| Complex32::from(x)).collect();
                samples.resize(len, Complex32::new(0.0, 0.0));

                let fft = fourier_transform::fft(&samples);
                let mut global_frequencies = CURRENT_FRAME_FREQUENCIES.lock().unwrap();
                for f in 0..len {
                    global_frequencies[f] = fft[f];
                }

                *FREQUENCY_COUNT.lock().unwrap() = len;
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
                let count = *FREQUENCY_COUNT.lock().unwrap();
                if count > 0 {
                    let cell_width = WIDTH as f32 / count as f32;
                    let mut max_amp = MAX_AMP.lock().unwrap();

                    let frequencies = *CURRENT_FRAME_FREQUENCIES.lock().unwrap();
                    for f in 0..count {
                        let fr = frequencies[f];
                        if fr > *max_amp {
                            *max_amp = fr
                        };

                        let t = fr / *max_amp;
                        let bar_height = HEIGHT as f32 / 2.0 * t;
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
                *volume = if *volume < VOLUME_CHANGE_BY {
                    MIN_VOLUME
                } else {
                    *volume - VOLUME_CHANGE_BY
                };
                music.set_volume(*volume);
            }

            KeyboardKey::KEY_UP => {
                println!("volume up");
                *volume = if *volume + VOLUME_CHANGE_BY < MAX_VOLUME {
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
