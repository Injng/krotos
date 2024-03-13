//use std::collections::VecDeque;
use raylib::prelude::*;
use awedio::Sound;
use std::f64::consts::{E, PI};
use std::ops::{Div, Mul};
use num_complex::{Complex, ComplexFloat};

const I:Complex<f64> = Complex::new(0.0, -2.0*PI);
fn main() {
    // initialize ray lib and awed io
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
    let frame_size: i32 = 512;//has to be a power of 2 for fft to work
    let width: i32 = handle.get_screen_width() / (frame_size-192);//optimization for the width of the line 46 is just a random number
    let screen_height: i32 = handle.get_screen_height();
    let mut drawings: Vec<f64> = Vec::new();
    for _ in 0..frame_size {
        drawings.push(0.0);
    }
    let update_time: u64 = (1000 / sample_rate) as u64;
    let mut updates: u64 = 1;

    unsafe {
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
                //drawings.pop_front();
                //drawings.push_back(sample.into());
                drawings.remove(0);
                drawings.push(sample.into());
                drawings = real_fft_filter(drawings.clone(), 0.01, 3.14, 0.1);//filter out the noise
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

// Define the size of the points to be plotted
            // Plot the points
            let mut prevx = 0.0;
            let mut prevy = 0.0;

            let mut prevmidx = 0.0;
            let mut prevmidy = 0.0;

         //  for (i, sample) in drawings.iter().enumerate() {
         //          print!("{} ", prevmidx);
         //          let x = i as f32 * width as f32; // Convert to f32 for accurate positioning
         //          let y = screen_height / 2; // Convert sample to f32 for y-coordinate
         //          let height = sample.clone() as f32 / 50.0;
         //          // Plot points at (x, y) with the specified color
         //          drawing.draw_line(prevx as i32, prevy as i32, x as i32, y + height as i32, Color::WHITE);
         //      prevx = x;
         //      prevy = y as f32 + height;
         //      ; // Convert sample to f32 for y-coordinate
         //   }
            for i in 0..drawings.len()-2{
                let x = i as f32 * width as f32; // Convert to f32 for accurate positioning
                let y = screen_height / 2; // Convert sample to f32 for y-coordinate
                let height0 = drawings[i] as f32 / 50.0;
                let height1 = drawings[i+1] as f32 / 50.0;
                let height2 = drawings[i+2] as f32 / 50.0;
                // Plot points at (x, y) with the specified color
                for t in 0..=100{
                    let t = t as f32 / 100.0;
                    let p0 = Vector2::new(x, y as f32+ height0);
                    let p1 = Vector2::new(x +width as f32, y as f32+ height1);
                    let p2 = Vector2::new(x+width as f32 *2.0, y as f32+ height2);
                    let p = quadratic_bezier(p0, p1, p2, t);
                    drawing.draw_pixel(p.x as i32, p.y as i32, Color::WHITE);
                }
            }
        }
    }
}
///
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

///
/// This is the inverse function of Fast Fourier Transform
///it transforms the frequency domain back to the real domain
///
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


// Quadratic bezier curve interpolation
fn quadratic_bezier(p0: Vector2, p1: Vector2, p2: Vector2, t: f32) -> Vector2 {
    let u = 1.0 - t;
    let u_squared = u * u;
    let t_squared = t * t;

    // Component-wise multiplication and addition
    let x = u_squared * p0.x + 2.0 * u * t * p1.x + t_squared * p2.x;
    let y = u_squared * p0.y + 2.0 * u * t * p1.y + t_squared * p2.y;

    Vector2::new(x, y)
}

fn real_fft_filter(arr:Vec<f64>, low:f64, high:f64, mag:f64) -> Vec<f64> {
    let n = arr.len();
    let mut ans: Vec<Complex<f64>> = vec![];
    for i in 0..n {
        ans.push(Complex::new(arr[i], 0.0));
    }
    return filter(ans, low, high, mag);
}
fn filter(mut arr: Vec<Complex<f64>>, low: f64, high: f64, mag1: f64) -> Vec<f64> {

    let n = arr.len();
    arr = fft(arr);
    let mut filter: Vec<Complex<f64>> = vec![];

    for i in 0..n {
        let freq = i as f64 / n as f64;
        if arr[i].re().abs() + arr[i].im().abs() < mag1 ||(freq < low && freq > high) {
            filter.push(Complex::new(0.0, 0.0));
        } else {
            filter.push(arr[i])
        }
    }
    filter = ifft(filter);
    let mut ans: Vec<f64> = vec![];
    for i in 0..n {
        ans.push(filter[i].re());
    }
    return ans;

}

