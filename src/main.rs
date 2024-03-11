//use std::collections::VecDeque;
use raylib::prelude::*;
use awedio::Sound;
use std::f64::consts::{E, PI};
use std::ops::{Div, Mul};
use num_complex::{Complex, ComplexFloat};
use raylib::ffi::{cos, sin};

const I:Complex<f64> = Complex::new(0.0, -2.0*PI);
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
    let frame_size: i32 = 128;//has to be a power of 2 for fft to work
    let width: i32 = handle.get_screen_width() / frame_size;
    let screen_height: i32 = handle.get_screen_height();
    let mut drawings: Vec<f64> = Vec::new(); //VecDeque<i16> = VecDeque::new();
    for _ in 0..frame_size {
        drawings.push(0.0);
    }
    let mut newdomain: Vec<Vec<f64>> = vec![];
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
                let frequency_domain_vector = real_fft(drawings.clone());
                let scalefactor = 4;
                newdomain = increase_resolution(frequency_domain_vector, scalefactor);
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
            //    let mut drawing: RaylibDrawHandle = handle.begin_drawing(&thread);
            //    RaylibDraw::clear_background(&mut drawing, Color::RAYWHITE);
            //    for (i, sample) in drawings.iter().enumerate() {
            //        let x = i as i32 * width;
            //        let y = screen_height / 2;
            //        let height = sample.clone() as i32 / 100;
            //        if height < 0 {
            //            drawing.draw_rectangle(x, y, width, -1 * height, Color::BLACK);
            //        } else {
            //            drawing.draw_rectangle(x, y - height, width, height, Color::BLACK);
            //        }
            let mut drawing: RaylibDrawHandle = handle.begin_drawing(&thread);
            RaylibDraw::clear_background(&mut drawing, Color::DARKGRAY);

// Define the size of the points to be plotted
            // Plot the points
            let mut prevx = 0;
            let mut prevy = 0;
            for (i, sample) in newdomain.iter().enumerate() {
                let x = sample[0] as f32 * width as f32; // Convert to f32 for accurate positioning
                let y = screen_height / 2; // Convert sample to f32 for y-coordinate
                let height = sample[1].clone() as i32 / 20;

                // Plot points at (x, y) with the specified color
                //drawing.draw_circle(x as i32, y as i32 +height, point_size, Color::WHITE);
                drawing.draw_line(prevx as i32, prevy as i32, x as i32, y as i32 + height, Color::WHITE);
                drawing.draw_pixel((x ) as i32, y as i32 + height, Color::WHITE);
                // Update the previous x and y coordinates
                prevx = x as i32;
                prevy = y as i32 + height;
                //print!("{} ", sample[1]);
            }
        }
    }
}
///
/// The funtion can only take input with length of powers of 2
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

fn increase_resolution(arr:Vec<Complex<f64>>, scalefactor:i32) -> Vec<Vec<f64>> {
    if scalefactor%2 != 0 {
        panic!("The scale factor should be a power of 2")
    }

    let mut ans: Vec<Vec<f64>> = vec![];
    let mut temp: Vec<Complex<f64>> = vec![];
    for j in 0..arr.len() {
        temp.push(arr[j]);
        for i in 0..scalefactor-1 {
            temp.push(Complex::new(0.0, 0.0));
        }
    }
    temp = ifft(temp);
    for j in 0..temp.len() {
        let mut temp2: Vec<f64> = vec![j as f64 /scalefactor as f64, 0.0];
        temp2[1] = temp[j].re;
        ans.push(temp2);
    }
    return ans;}

//   fn increaes_resolution(amplitude: Vec<f64>, phase: Vec<f64>, newsize:i32) -> Vec<Vec<f64>> {
//       if phase.len() != amplitude.len() {
//           panic!("The length of the frequency and phase should be the same")
//       }
//       let factor = phase.len() as f64 / newsize  as f64;
//       let mut ans: Vec<Vec<f64>> = vec![];
//       for j in 0..newsize {
//           let mut temp: Vec<f64> = vec![j as f64 * factor, 0.0];
//           for i in 0..phase.len() {
//               temp[1]+=amplitude[i]*(phase[i].sin() + phase[i].cos());;
//           }
//           temp[1]/=newsize as f64;
//           //print!("{:?} ", temp.clone());
//           ans.push(temp);
//
//       }
//
//       return ans;
//   }
fn real_fft(arr:Vec<f64>) -> Vec<Complex<f64>> {
    let n = arr.len();
    let mut ans: Vec<Complex<f64>> = vec![];
    for i in 0..n {
        ans.push(Complex::new(arr[i], 0.0));
    }
    return fft(ans);
}

