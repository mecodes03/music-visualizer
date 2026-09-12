use num::complex::Complex32;
use raylib::prelude::*;
use std::sync::Mutex;

mod fourier_transform;

// TODO: use array everywhere instead of vec since most lengts(kinda estimated) are known at compile time.

static FREQUENCY_COUNT: Mutex<usize> = Mutex::new(0);
static CAPACITY: usize = 1024;
/** samples in one Frame of 60 FPS*/
static CURRENT_FRAME_FREQUENCIES: Mutex<[f32; CAPACITY]> = Mutex::new([0.0; CAPACITY]);

/** Programme using safe raylib rust bindings. */
fn main() {
    const WIDTH: i32 = 640;
    const HEIGHT: i32 = 480;

    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Musializer")
        .build();
    rl.set_target_fps(60);

    let raylib_audio_device = RaylibAudio::init_audio_device().expect("audio init failed");
    let music = raylib_audio_device
        .new_music("audio.mp3")
        .expect("sound load failed");
    music.play_stream();

    let mut audio_stream_processor = move |_samples: &mut [f32], _channels: u32| {
        let samples: Vec<Complex32> = Vec::from(_samples)
            .iter()
            .step_by(2)
            .map(|&n| Complex32::from(n))
            .collect();

        let fft = fourier_transform::fft(&samples);
        let mut global_frequencies = CURRENT_FRAME_FREQUENCIES.lock().unwrap();

        let len = fft.len();
        for f in 0..len {
            global_frequencies[f] = fft[f];
        }

        *FREQUENCY_COUNT.lock().unwrap() = len;
    };

    let _guard = attach_audio_stream_processor_to_music(&music, &mut audio_stream_processor);

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        let frequency_count = *FREQUENCY_COUNT.lock().unwrap();
        if frequency_count > 0 {
            let frequencies = *CURRENT_FRAME_FREQUENCIES.lock().unwrap();

            let cell_width = WIDTH as f32 / frequency_count as f32;
            let cell_height = HEIGHT as f32 / frequency_count as f32;

            for f in 0..frequency_count {
                let bar_height = frequencies[f] * cell_height;
                let rect = Rectangle::new(
                    f as f32 * cell_width,
                    HEIGHT as f32 / 2.0 - bar_height,
                    cell_width,
                    bar_height,
                );
                d.draw_rectangle_rec(rect, color::Color::WHITE);
            }
        }

        music.update_stream();
    }
}
