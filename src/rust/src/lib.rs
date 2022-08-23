use extendr_api::prelude::*;
use std::path::Path;

mod decode_symphonia;
mod misc;
mod play_audio;

/// Load an audio file as an array of doubles.
/// @param fname \[String\] The path to the input file. 
/// @param mono \[Bool\] Convert the audio to mono, taking the average across channels.
/// @param offset \[Double\] Start reading the file after the offset, in seconds.
/// @param duration \[Double\] Duration to be loaded, in seconds, counting from the offset. Will load the file till the end if offset + duration >= file length. 
/// @return a 2D (nsamples, channels) array of doubles. The samples are normalized to fit in the range of \[-1.0, 1.0\].
/// @examples 
/// load("test", FALSE, 1, 2, "symphonia")
/// load("test")
/// @export
#[extendr]
pub fn load(
    fname: &str,
    #[default = "TRUE"] mono: bool,
    #[default = "0."] offset: f64,
    #[default = "NA_real_"] duration: Option<f64>,
) -> Robj {
    let path = Path::new(fname);
    let filetype = Path::extension(path)
        .expect("couldn't extract the file extension")
        .to_str()
        .expect("cannot convert from &OsStr to &str");

    let decoded_arr = decode_symphonia::load(path, mono, offset, duration, filetype);

    Robj::try_from(&decoded_arr.t()).expect("cannot convert ndarray to Robj") // try to return a matrix or Rarr instead of Robj
}


/// Convert to 1 channel taking the average across channels.
/// @param r_arr \[Matrix\] Matrix of doubles representing the audio data. 
/// @return a 2D (samples x 1) array of doubles.
/// @examples 
/// x <- array(c(1,2,3,4), c(2, 2))
/// to_mono(x)
/// @export
#[extendr]
pub fn to_mono(r_arr: RMatrix<f64>) -> Robj {
    let arr: ArrayView2<f64> =
        ArrayView2::from_robj(&r_arr).expect("cannot convert Robj to ArrayView2");
    let arr_mono: Array2<f64> = misc::to_mono_ndarray(&arr.t());

    Robj::try_from(&arr_mono.t()).expect("cannot convert ndarray to Robj") // try to return a matrix or Rarr instead of Robj
}

/// Get the audio duration in seconds. \cr
/// At least one of (`r_arr`, `sr`), `s` or `fname` should be provided. 
/// @param r_arr \[Matrix\] Matrix of doubles (nsamples, channels) representing the audio data. 
/// @param sr \[Integer\] Audio sampling rate.
/// @param s \[Array\] a 3D (t, f, channels) array of complexes representing a STFT or any STFT-derived matrix (e.g., chromagram or mel spectrogram). 
/// @param n_fft \[Integer\] FFT window size for `s`. 
/// @param hop_length \[Integer\] Number of audio samples between columns of `s`.
/// @param center \[bool\] 
/// \itemize{
/// \item If `TRUE`, `s[t, , ]` is centered at `r_arr[t * hop_length, ]`
/// \item If `FALSE`, then `s[t, , ]` begins at `r_arr[t * hop_length, ]`
/// }
/// @param fname \[String\] The path to the input file. If provided, all other parameters are ignored, and the duration is calculated directly from the audio file. Note that this avoids loading the contents into memory, and is therefore useful for querying the duration of long files.
/// @section Notes:
/// `get_duration` can be applied to a file (`fname`), a spectrogram (`s`), or audio buffer (`r_arr`, `sr`).  At least one of these three options should be provided.  If you do provide multiple options (e.g., `fname` and `s`), then `fname` takes precedence over `s`, and `s` takes precedence over (`y`, `sr`).
/// @return a double.
/// @examples 
/// x <- array(c(1,2,3,4), c(2, 2))
/// to_mono(x)
/// @export
#[extendr]
pub fn get_duration(
    // fix arguments.
    #[default = "NULL"] r_arr: Robj, 
    #[default = "22050L"] sr: i32, // sr had to be i32 so can add default argument. R works with i32.
    #[default = "NULL"] s: Robj, 
    #[default = "2048L"] n_fft: i32, // sr had to be i32 so can add default argument. R works with i32.
    #[default = "512L"] hop_length: i32, // sr had to be i32 so can add default argument. R works with i32.
    #[default = "TRUE"] center: bool, // sr had to be i32 so can add default argument. R works with i32.
    #[default = "NA_character_"] fname: Option<&str>,
) -> f64 {
    match (r_arr.is_null(), fname, s.is_null()) {
        (_, Some(fname_), _) => {
            let path = Path::new(fname_);
            let filetype = Path::extension(path)
                .expect("couldn't extract the file extension")
                .to_str()
                .expect("cannot convert from &OsStr to &str");

            decode_symphonia::get_duration(path, filetype)
        },
        (_, None, false) => {
            if sr <= 0 { panic!("sr must be positive"); }
            if n_fft <= 0 { panic!("n_fft must be positive"); }
            if hop_length <= 0 { panic!("hop_length must be positive"); }
            let dim: Vec<Rint> = s.dim().expect("cannot get dimensions").iter().collect();
            if dim.len() != 3 || !s.is_array() { panic!("s must be a 3D array") };

            let n_frames = dim[0].0;
            let mut n_samples = n_fft + hop_length * (n_frames - 1);

            // if centered, we lose half a window from each end of s
            if center {
                n_samples -= 2 * (n_fft / 2);
            }

            n_samples as f64 / sr as f64

        },
        (false, None, true) => {
            if !r_arr.is_matrix() { panic!("r_arr must be a matrix"); }
            if sr <= 0 { panic!("sr must be positive"); }

            r_arr.nrows() as f64 / sr as f64 // need to fix conversion for r_arr.nrows() since usize to f64 is not safe.
        },
        (_, _, _) => panic!("At least one of (r_arr, sr), s, fname should be provided"),
    }
}

/// Get the audio sampling rate.
/// @param fname \[String\] The path to the input file. 
/// @return an integer.
/// @examples 
/// get_samplerate("test")
/// get_samplerate("test.flac", "claxon")
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
    let arr: ArrayView2<f64> = ArrayView2::from_robj(&r_arr).expect("cannot convert Robj to ArrayView2");
    play_audio::play(&arr, sr as u32)
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod audiotest;
    fn load;
    fn to_mono;
    fn get_duration;
    fn get_samplerate;
    fn play;
}

