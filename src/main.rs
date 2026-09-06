use raylib::ffi::{
    AttachAudioStreamProcessor, BeginDrawing, ClearBackground, Color, DrawRectangleRec, EndDrawing,
    GetKeyPressed, GetMusicTimeLength, GetMusicTimePlayed, GetRenderHeight, GetRenderWidth,
    InitAudioDevice, InitWindow, IsMusicStreamPlaying,
    KeyboardKey::{self},
    LoadMusicStream, PauseMusicStream, PlayMusicStream, Rectangle, ResumeMusicStream,
    SeekMusicStream, SetMusicVolume, SetTargetFPS, UpdateMusicStream, WindowShouldClose,
};

use std::ffi::CString;
use std::os::raw::c_void;
use std::sync::Mutex;

// constants
const WINDOW_WIDTH: i32 = 500;
const WINDOW_HEIGHT: i32 = 300;
const MAX_VOLUME: f32 = 1.0;
const VOLUME_CHANGE_BY: f32 = 0.05;
const INITIAL_VOLUME: f32 = 0.5;
const SEEK_BY: f32 = 5.0;

#[derive(Copy, Clone)]
struct Frame {
    left: f32,
    right: f32,
}

const GLOBAL_FRAME_CAPACITY: usize = 1024;
static GLOBAL_FRAMES: Mutex<[Frame; GLOBAL_FRAME_CAPACITY]> = Mutex::new(
    [Frame {
        left: 0.0,
        right: 0.0,
    }; 1024],
);
static GLOBAL_FRAME_COUNT: Mutex<u32> = Mutex::new(0);
static CHANNELS: Mutex<u32> = Mutex::new(1);

fn main() {
    let file_name = CString::new("audio.mp3").unwrap();

    unsafe {
        let title = CString::new("Musializer").unwrap();
        InitWindow(WINDOW_WIDTH, WINDOW_HEIGHT, title.as_ptr());
        SetTargetFPS(60);

        InitAudioDevice();

        let music = LoadMusicStream(file_name.as_ptr());
        println!("sampleRate: {}", music.stream.sampleRate);
        println!("sampleSize: {}", music.stream.sampleSize);
        println!("channels: {}", music.stream.channels);
        println!("Tadow!!!!!!!!!!!!!!!!!!");

        *CHANNELS.lock().unwrap() = music.stream.channels;

        PlayMusicStream(music);

        let mut volume: f32 = INITIAL_VOLUME;
        SetMusicVolume(music, volume);

        AttachAudioStreamProcessor(music.stream, Some(audio_callback));

        let h = GetRenderHeight();
        let w = GetRenderWidth();

        let mut current_key: i32;
        while !WindowShouldClose() {
            UpdateMusicStream(music);

            current_key = GetKeyPressed();
            handle_keyboard(music, current_key, &mut volume);

            BeginDrawing();
            ClearBackground(Color::BLACK);

            let snapshot = *GLOBAL_FRAMES.lock().unwrap();
            let frame_count = *GLOBAL_FRAME_COUNT.lock().unwrap();

            if frame_count > 0 {
                let cell_width = w as f32 / (frame_count as f32);

                for i in 0..frame_count {
                    let sample_l = snapshot[i as usize].left;
                    let bar_height = (h as f32 / 2.0) * sample_l.abs();
                    let rect = if sample_l > 0.0 {
                        Rectangle::new(
                            i as f32 * cell_width,
                            (h as f32 / 2.0) - bar_height,
                            cell_width,
                            bar_height * 0.1,
                        )
                    } else {
                        Rectangle::new(
                            i as f32 * cell_width,
                            h as f32 / 2.0 + bar_height,
                            cell_width,
                            bar_height * 0.05,
                        )
                    };
                    // println!("{:?}", rect);
                    DrawRectangleRec(rect, Color::RED);
                }
            }
            EndDrawing();
        }
    };
}

fn handle_keyboard(music: raylib::prelude::ffi::Music, current_key: i32, volume: &mut f32) {
    match current_key {
        key if key == KeyboardKey::KEY_H as i32 => {
            let current_time_played = unsafe { GetMusicTimePlayed(music) };
            let seek = if current_time_played <= SEEK_BY {
                0.0
            } else {
                current_time_played - SEEK_BY
            };
            unsafe { SeekMusicStream(music, seek) };
        }

        key if key == KeyboardKey::KEY_L as i32 => {
            let current_time_played = unsafe { GetMusicTimePlayed(music) };
            let music_length = unsafe { GetMusicTimeLength(music) };
            let seek = if current_time_played + SEEK_BY > music_length {
                music_length
            } else {
                current_time_played + SEEK_BY
            };
            unsafe { SeekMusicStream(music, seek) };
        }

        key if key == KeyboardKey::KEY_SPACE as i32 => {
            if unsafe { IsMusicStreamPlaying(music) } {
                unsafe { PauseMusicStream(music) };
            } else {
                unsafe { ResumeMusicStream(music) };
            }
        }

        key if key == KeyboardKey::KEY_DOWN as i32 => {
            println!("volume down");
            *volume = if *volume < 0.1 { 0.0 } else { *volume - 0.1 };
            unsafe { SetMusicVolume(music, *volume) };
        }

        key if key == KeyboardKey::KEY_UP as i32 => {
            println!("volume up");
            *volume = if *volume < 0.9 {
                *volume + VOLUME_CHANGE_BY
            } else {
                MAX_VOLUME
            };
            unsafe { SetMusicVolume(music, *volume) };
        }
        _ => {}
    }
}

unsafe extern "C" fn audio_callback(buffer: *mut c_void, frames: u32) {
    let gf_count = *GLOBAL_FRAME_COUNT.lock().unwrap() as usize;
    let frame_count = (frames as usize).min(if gf_count > 0 {
        gf_count
    } else {
        frames as usize
    });
    let samples = unsafe {
        std::slice::from_raw_parts(
            buffer as *const f32,
            frame_count * (*CHANNELS.lock().unwrap() as usize),
        )
    };
    let mut global = GLOBAL_FRAMES.lock().unwrap();
    for i in 0..frame_count {
        global[i as usize].left = samples[2 * i];
        global[i as usize].right = samples[2 * i + 1]
    }
    *GLOBAL_FRAME_COUNT.lock().unwrap() = frame_count as u32;
}
