use raylib::ffi::{
    AttachAudioStreamProcessor, BeginDrawing, ClearBackground, Color, DrawRectangleRec, EndDrawing,
    GetKeyPressed, GetMusicTimeLength, GetMusicTimePlayed, GetRenderHeight, GetRenderWidth,
    InitAudioDevice, InitWindow, IsMusicStreamPlaying,
    KeyboardKey::{self},
    LoadMusicStream, PauseMusicStream, PlayMusicStream, Rectangle, ResumeMusicStream,
    SeekMusicStream, SetMusicVolume, SetTargetFPS, UpdateMusicStream, WindowShouldClose,
};

use std::os::raw::c_void;
use std::{ffi::CString, sync::Mutex};

// TODO: try to make this program only using safe rust.

const WINDOW_WIDTH: i32 = 400;
const WINDOW_HEIGHT: i32 = 300;

const MAX_VOLUME: f32 = 1.0;
const VOLUME_CHANGE_BY: f32 = 0.05;
const INITIAL_VOLUME: f32 = 0.5;
const SEEK_BY: f32 = 5.0;

const GLOBAL_FRAME_COUNT: usize = 1024;
static GLOBAL_FRAMES: Mutex<[f32; GLOBAL_FRAME_COUNT]> = Mutex::new([0.0; GLOBAL_FRAME_COUNT]);

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

        PlayMusicStream(music);

        let mut volume: f32 = INITIAL_VOLUME;
        SetMusicVolume(music, volume);

        AttachAudioStreamProcessor(music.stream, Some(audio_callback));

        let h = GetRenderHeight();
        let w = GetRenderWidth();
        let cell_width = w as f32 / (GLOBAL_FRAME_COUNT as f32);
        println!("cell_width: {}", cell_width);

        let mut current_key: i32;
        while !WindowShouldClose() {
            UpdateMusicStream(music);

            current_key = GetKeyPressed();
            handle_keyboard(music, current_key, &mut volume);

            BeginDrawing();
            ClearBackground(Color::BLACK);

            let mut i = 0;
            for frame in GLOBAL_FRAMES.lock().unwrap().iter() {
                let bar_height = (h as f32 / 2.0) * frame.abs();
                let rect = if *frame > 0.0 {
                    Rectangle::new(
                        i as f32 * cell_width,
                        (h as f32 / 2.0) - bar_height,
                        cell_width,
                        bar_height,
                    )
                } else {
                    Rectangle::new(
                        i as f32 * cell_width,
                        h as f32 / 2.0,
                        cell_width,
                        bar_height,
                    )
                };
                DrawRectangleRec(rect, Color::RED);
                i += 1;
            }

            EndDrawing();
        }
    };

    println!("Tadow!!!!!!!!!!!!!!!!!!");
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
    let sample_count = ((frames as usize) * 2).min(GLOBAL_FRAME_COUNT);
    let samples =
        unsafe { std::slice::from_raw_parts(buffer as *const f32, sample_count as usize) };
    GLOBAL_FRAMES.lock().unwrap()[..sample_count].copy_from_slice(samples);
}
