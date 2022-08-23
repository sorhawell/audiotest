use extendr_api::prelude::*;
use std::path::Path;

mod decode_symphonia;
mod misc;
mod play_audio;

use play_audio::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Sample, SampleRate, StreamConfig};
use ndarray::ArrayView2;

#[extendr]
pub fn test_in_R() {
    let fname = "./test_files/mono.wav";
    let path = Path::new(fname);
    let filetype = Path::extension(path)
        .expect("couldn't extract the file extension")
        .to_str()
        .expect("cannot convert from &OsStr to &str");
    let decoded_arr = decode_symphonia::load(path, false, 0., NA_REAL, filetype);
    let sr = decode_symphonia::get_samplerate(path, filetype);
    rprintln!("{:?}", decoded_arr);
    rprintln!("{:?}", sr);
    play_audio::play(&decoded_arr.view(), sr);
}

/// @export
#[extendr]
pub fn load(
    fname: &str,
    mono: bool,            // #[default = "TRUE"]
    offset: f64,           //#[default = "0."]
    duration: Option<f64>, // #[default = "NA_real_"]
) -> Robj {
    let path = Path::new(fname);
    let filetype = Path::extension(path)
        .expect("couldn't extract the file extension")
        .to_str()
        .expect("cannot convert from &OsStr to &str");

    let decoded_arr = decode_symphonia::load(path, mono, offset, duration, filetype);

    Robj::try_from(&decoded_arr.t()).expect("cannot convert ndarray to Robj") // try to return a matrix or Rarr instead of Robj
}

#[extendr]
pub struct ArrayBaseR(pub ArrayBase<OwnedRepr<f64>, Dim<[usize; 2]>>);

#[extendr]
impl ArrayBaseR {
    pub fn print(&self) {
        rprintln!("imma arraaaayy argh \n {:?}", self.0);
    }
}

/// @export
#[extendr]
pub fn load2(
    fname: &str,
    mono: bool,            // #[default = "TRUE"]
    offset: f64,           //#[default = "0."]
    duration: Option<f64>, // #[default = "NA_real_"]
) -> ArrayBaseR {
    let path = Path::new(fname);
    let filetype = Path::extension(path)
        .expect("couldn't extract the file extension")
        .to_str()
        .expect("cannot convert from &OsStr to &str");

    let decoded_arr = decode_symphonia::load(path, mono, offset, duration, filetype);

    ArrayBaseR(decoded_arr)
}

/// @export
#[extendr]
pub fn get_samplerate(fname: &str) -> i32 {
    let path = Path::new(fname);
    let filetype = Path::extension(path)
        .expect("couldn't extract the file extension")
        .to_str()
        .expect("cannot convert from &OsStr to &str");

    let sr = decode_symphonia::get_samplerate(path, filetype);

    i32::try_from(sr).expect("cannot convert u32 to i32.")
}

/// @export
#[extendr]
pub fn play(r_arr: RMatrix<f64>, sr: i32) {
    let robj = RMatrix::into_robj(r_arr);
    let arr: ArrayView2<f64> =
        ArrayView2::from_robj(&robj).expect("cannot convert Robj to ArrayView2");

    println!("{:?}", arr);
    play_audio::play(&arr, sr as u32)
}

/// @export
#[extendr]
pub fn play2(abar: &ArrayBaseR, sr: i32) {
    let x = abar.0.clone();
    play_audio::play(&x.view(), sr as u32)
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod audiotest;
    fn load;
    // fn to_mono;
    // fn get_duration;
    fn get_samplerate;
    fn play;
    fn test_in_R;
    impl ArrayBaseR;
    fn load2;
    fn play2;
}
