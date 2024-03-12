use raylib::prelude::*;
use awedio::Sound;
use std::f64::consts::{E, PI};
use std::ops::{Div, Mul};
use num_complex::{Complex, ComplexFloat};

const I:Complex<f64> = Complex::new(0.0, -2.0*PI);

fn main() {
    // initialize ray lib and awedio
    let (mut handle, thread) = init().build();
    // init raylib music
    let mut audio = RaylibAudio::init_audio_device();
    let mut music = Music::load_music_stream(&thread, "src/interconnectedness.ogg").unwrap();
    RaylibAudio::play_music_stream(&mut audio, &mut music);
    // load sound for samples
    let mut samples = awedio::sounds::open_file("src/interconnectedness.ogg").unwrap();

    // initialize variables
    let mut is_paused: bool = false;
    let sample_rate: u32 = samples.sample_rate();
    let frame_size: i32 = 256; // must be power of 2
    // optimization for the width of the line 46 is just a random number
    let width: i32 = handle.get_screen_width() / (frame_size-46);
    let screen_height: i32 = handle.get_screen_height();
    let mut drawings: Vec<f64> = Vec::new();
    for _ in 0..frame_size {
        drawings.push(0.0);
    }
    let update_time: u64 = (1000 / sample_rate) as u64;
    let mut updates: u64 = 1;

    while !handle.window_should_close() {
        // every update_time, update drawings with new sample
        RaylibAudio::update_music_stream(&mut audio, &mut music);
        let time_played = RaylibAudio::get_music_time_played(&audio, &music);
        if !is_paused && (time_played * 1024.0) as u64 > update_time * updates {
            updates += 1;
            let next_frame = samples.next_frame().unwrap();
            let mut sample = next_frame[0];
            if (sample as i32).abs() < 1024 {
                sample = 0;
            }
            drawings.remove(0);
            drawings.push(sample.into());

            // filter out noise
            drawings = real_fft_filter(drawings.clone(), 0.01, 0.9);         
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

        let mut drawing: RaylibDrawHandle = handle.begin_drawing(&thread);
        RaylibDraw::clear_background(&mut drawing, Color::DARKGRAY);

        // define the size of the points to be plotted
        // plot the points
        let mut prevx = 0.0;
        let mut prevy = 0.0;
        for (i, sample) in drawings.iter().enumerate() {
            let x = i as f32 * width as f32; 
            let y = screen_height / 2; 
            let height = sample.clone() as f32 / 50.0;

            // plot points at (x, y) with the specified color
            drawing.draw_line(prevx as i32, prevy as i32, x as i32, y + height as i32, Color::WHITE);
            prevx = x;
            prevy = y as f32 + height;
        }
    }
    }
}

/// The function can only take input with length of powers of 2
/// It will return a list of complex numbers as the result of Fast Fourier Transform
/// It provides you with amplitude and phase
/// The amplitude is enclosed as the magnitude of the complex number sqrt(x^2 + y^2)
/// The phase is enclosed as the angle of the complex number atan2(y,x)
fn fft(arr: Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    let n = arr.len();
    if n<=1{
        return arr.clone();
    }
    let mut even: Vec<Complex<f64>> = vec![];
    let mut odd : Vec<Complex<f64>> = vec![];
    for i in 0..n {
        if i%2 == 0 {
            even.push(arr[i])
        }else {
            odd.push(arr[i])
        }
    }
    even = fft(even);
    odd = fft(odd);
    let mut t: Vec<Complex<f64>> = vec![];
    let mut i:Complex<f64> = Complex:: new(1.0, 0.0);
    let exp = E.powc(I.mul(1.0/n as f64));

    for j in 0..n/2 {
        t.push( i* odd[j]);
        i*=exp;
    }
    let mut ans: Vec<Complex<f64>> = vec![];
    for i in 0..n/2 {
        ans.push(even[i] + t[i])
    }
    for i in 0..n/2 {
        ans.push(even[i]- t[i])
    }
    return ans
}

/// This is the inverse function of Fast Fourier Transform
/// It transforms the frequency domain back to the real domain
fn ifft(arr: Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    let mut ans : Vec<Complex<f64>> = vec![];
    let n = arr.len();
    for i in  0..n {
        ans.push(arr[i].conj());
    }
    ans = fft(ans);
    for i in 0..n {
        ans[i]=ans[i].div(n as f64);
    }
    return ans;
}

fn real_fft_filter(arr:Vec<f64>, low:f64, high:f64) -> Vec<f64> {
    let n = arr.len();
    let mut ans: Vec<Complex<f64>> = vec![];
    for i in 0..n {
        ans.push(Complex::new(arr[i], 0.0));
    }
    return filter(ans, low, high);
}

fn filter(mut arr: Vec<Complex<f64>>, low: f64, high: f64) -> Vec<f64> {
    let n = arr.len();
    arr = fft(arr);
    let mut filter: Vec<Complex<f64>> = vec![];

    for i in 0..n {
        let freq = i as f64 / n as f64;
        if freq >= low && freq <= high {
            filter.push(arr[i]);
        } else {
            filter.push(Complex::new(0.0, 0.0));
        }
    }
    filter = ifft(filter);
    let mut ans: Vec<f64> = vec![];
    for i in 0..n {
        ans.push(filter[i].re());
    }
    return ans;
}
