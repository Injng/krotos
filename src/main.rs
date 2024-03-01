use std::collections::VecDeque;
use raylib::prelude::*;
use awedio::Sound;

fn main() {
    // initialize raylib and awedio
    let (mut handle, thread) = init().build();
    // init raylib music
    let mut audio = RaylibAudio::init_audio_device();
    let mut music = Music::load_music_stream(&thread, "src/interconnectedness.ogg").unwrap();
    RaylibAudio::play_music_stream(&mut audio, &mut music);
    // load sound for samples
    let mut samples = awedio::sounds::open_file("src/interconnectedness.ogg")
        .unwrap();

    // initialize variables
    let mut is_paused: bool = false;
    let sample_rate: u32 = samples.sample_rate();
    let frame_size: i32 = 200;
    let width: i32 = handle.get_screen_width() / frame_size;
    let screen_height: i32 = handle.get_screen_height();
    let mut drawings: VecDeque<i16> = VecDeque::new();
    for _ in 0..frame_size {
        drawings.push_back(0);
    }
    let update_time: u64 = (1000 / sample_rate) as u64;
    let mut updates: u64 = 1;

    while !handle.window_should_close() {
        // every update_time, update drawings with new sample
        RaylibAudio::update_music_stream(&mut audio, &mut music);
        let time_played = RaylibAudio::get_music_time_played(&audio, &music);
        if !is_paused && (time_played * 1000.0) as u64 > update_time * updates {
            updates += 1;
            let next_frame = samples.next_frame().unwrap();
            let mut sample = next_frame[0];
            if (sample as i32).abs() < 1000 {
                sample = 0;
            }
            drawings.pop_front();
            drawings.push_back(sample.into());
        }

        // handle pausing
        if handle.is_key_pressed(KeyboardKey::KEY_SPACE) {
            if is_paused {
                RaylibAudio::play_music_stream(&mut audio, &mut music);
                is_paused = false;
            } else {
                RaylibAudio::pause_music_stream(&mut audio, &mut music);
                is_paused = true;
            }
        }

        // draw the visualization
        let mut drawing: RaylibDrawHandle = handle.begin_drawing(&thread);
        RaylibDraw::clear_background(&mut drawing, Color::RAYWHITE);
        for (i, sample) in drawings.iter().enumerate() {
            let x = i as i32 * width;
            let y = screen_height / 2;
            let height = sample.clone() as i32 / 100;
            if height < 0 {
                drawing.draw_rectangle(x, y, width, -1 * height, Color::BLACK);
            } else {
                drawing.draw_rectangle(x, y - height, width, height, Color::BLACK);
            }
        }
    }
}
