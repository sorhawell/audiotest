use std::convert::TryFrom;
use std::fs::File;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use ndarray::{Array2, ArrayView2};

use crate::misc;

pub fn load(
    path: &Path,
    mono: bool,
    offset: f64,
    duration: Option<f64>,
    filetype: &str,
) -> Array2<f64> {
    // Create a media source. Note that the MediaSource trait is automatically implemented for File, among other types.
    let file = Box::new(File::open(path).expect("cannot open file"));
    // Create the media source stream using the boxed media source from above.
    let mss = MediaSourceStream::new(file, Default::default());
    // Create a hint to help the format registry guess what format reader is appropriate.
    let mut hint = Hint::new();
    hint.with_extension(filetype);
    // Use the default options when reading and decoding.
    let format_opts: FormatOptions = Default::default();
    let metadata_opts: MetadataOptions = Default::default();
    let decoder_opts: DecoderOptions = Default::default();
    // Probe the media source stream for a format.
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .expect("unsupported format");
    // Get the format reader yielded by the probe operation.
    let mut format = probed.format;
    // Get the default track.
    let track = format.default_track().expect("cannot get default_track");
    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .expect("cannot create decoder");
    let channels = decoder
        .codec_params()
        .channels
        .expect("cannot retrieve the number of channels")
        .count();
    let n_frames = decoder
        .codec_params()
        .n_frames
        .expect("cannot retrieve the number of frames"); // In PCM n_frames is the same as n_samples, but for each channel
    let sr = decoder
        .codec_params()
        .sample_rate
        .expect("cannot retrieve the sample rate");
    let file_time_duration = n_frames as f64 / sr as f64; // fix sample_per_channel conversion.
    let duration_to_decode = f64::min(
        // assures duration max = duration from file - offset
        duration.unwrap_or(file_time_duration - offset),
        file_time_duration - offset,
    );

    if duration_to_decode <= 0. {
        panic!("duration must be a positive number")
    }

    let mut offset_samples = (offset * (sr as f64)) as u32; // Round to the lower bound integer by default. fix conversion // offset_samples is by channel

    if (offset_samples as u64) >= n_frames {
        panic!("offset bigger than or equal to total duration");
    }

    let mut duration_to_decode_samples = (duration_to_decode * (sr as f64)) as u32; // Round to the lower bound integer by default. fix conversion
                                                                                    // Store the track identifier, we'll use it to filter packets.
    let track_id = track.id;
    let mut sample_buf = None;
    let mut arr = Array2::<f64>::zeros((channels, duration_to_decode_samples as usize));
    let mut idx = 0_usize;

    'outer: loop {
        // Get the next packet from the format reader.
        let packet = match format.next_packet() {
            Ok(packet_ok) => packet_ok,
            Err(Error::IoError(ref packet_err))
                if packet_err.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(packet_err) => panic!("{:?}", packet_err),
        };

        // If the packet does not belong to the selected track, skip it.
        if packet.track_id() != track_id {
            continue;
        }

        // Decode the packet into audio samples, ignoring any decode errors.
        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                // The decoded audio samples may now be accessed via the audio buffer if per-channel
                // slices of samples in their native decoded format is desired. Use-cases where
                // the samples need to be accessed in an interleaved order or converted into
                // another sample format, or a byte buffer is required, are covered by copying the
                // audio buffer into a sample buffer or raw sample buffer, respectively. In the
                // example below, we will copy the audio buffer into a sample buffer in an
                // interleaved order while also converting to a f64 sample format.

                // If this is the *first* decoded packet, create a sample buffer matching the
                // decoded audio buffer format.
                if sample_buf.is_none() {
                    // Get the audio buffer specification.
                    let spec = *audio_buf.spec();
                    // Get the capacity of the decoded buffer.
                    let cap = audio_buf.capacity() as u64;

                    // Create the f64 sample buffer.
                    sample_buf = Some(SampleBuffer::<f64>::new(cap, spec));
                }

                // Copy the decoded audio buffer into the sample buffer in an interleaved format.
                if let Some(buf) = &mut sample_buf {
                    buf.copy_interleaved_ref(audio_buf);

                    // The samples may now be access via the `samples()` function.
                    let mut samples = buf.samples();
                    let mut ch: usize;
                    let frames_in_block = u32::try_from(samples.len())
                        .expect("cannot safely convert u64 to u32")
                        / channels as u32;

                    if offset_samples >= frames_in_block {
                        // deal with offset
                        offset_samples -= frames_in_block;
                        continue;
                    } else if offset_samples != 0 {
                        samples = &samples[(offset_samples as usize) * channels..];
                        offset_samples = 0;
                    }

                    for (n, sample) in samples.iter().enumerate() {
                        ch = n % channels;
                        arr[[ch, idx]] = *sample;

                        if ch == channels - 1 {
                            idx += 1;
                            duration_to_decode_samples -= 1; // deal with duration_to_decode
                        }

                        if duration_to_decode_samples == 0 {
                            // then skip the rest
                            break 'outer;
                        }
                    }
                }
            }
            Err(Error::DecodeError(err_str)) => panic!("{}", err_str),
            Err(_) => break,
        }
    }

    if mono {
        arr = misc::to_mono_ndarray(&ArrayView2::from(&arr)); // use ArrayView so to_mono_ndarray only creates 1 copy.
    }

    arr
}

// pub fn get_duration(path: &Path, filetype: &str) -> f64 {
//     let file = Box::new(File::open(path).expect("cannot open file"));

//     let mss = MediaSourceStream::new(file, Default::default());

//     let mut hint = Hint::new();
//     hint.with_extension(filetype);

//     let format_opts: FormatOptions = Default::default();
//     let metadata_opts: MetadataOptions = Default::default();

//     let probed = symphonia::default::get_probe()
//         .format(&hint, mss, &format_opts, &metadata_opts)
//         .expect("unsupported format");

//     let format = probed.format;

//     let track = format.default_track().expect("cannot get default_track");

//     let sr = track
//         .codec_params
//         .sample_rate
//         .expect("cannot retrieve the sample rate");

//     let n_frames = track
//         .codec_params
//         .n_frames
//         .expect("cannot retrieve n_frames");

//     n_frames as f64 / (sr as f64) // fix n_frames conversion
// }

pub fn get_samplerate(path: &Path, filetype: &str) -> u32 {
    let file = Box::new(File::open(path).expect("cannot open file"));

    let mss = MediaSourceStream::new(file, Default::default());

    let mut hint = Hint::new();
    hint.with_extension(filetype);

    let format_opts: FormatOptions = Default::default();
    let metadata_opts: MetadataOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .expect("unsupported format");

    let format = probed.format;

    let track = format.default_track().expect("cannot get default_track");

    track
        .codec_params
        .sample_rate
        .expect("cannot retrieve the sample rate")
}

//pub fn stream(
//    path: &Path,
//    block_length: i32,
//    frame_length: i32,
//    hop_length: i32,
//    mono: bool,
//    offset: f64,
//    duration: Option<f64>,
//) -> Array2<f64> {
//    let v = vec![1,2,3];
//    let it = v.into_iter();
//
//
//    arr
//}
