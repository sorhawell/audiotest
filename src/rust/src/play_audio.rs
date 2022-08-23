pub use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
pub use cpal::{BufferSize, Sample, SampleRate, StreamConfig};
pub use ndarray::ArrayView2;

pub fn play(arr: &ArrayView2<f64>, sr: u32) {
    let channels = arr.nrows();
    let samples = arr.ncols();

    // convert to interleaved
    let mut data_interleaved = Vec::with_capacity(channels * samples);
    for i in 0..samples {
        for ch in 0..channels {
            data_interleaved.push(arr[[ch, i]] as f32);
        }
    }

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    //let mut supported_configs_range = device.supported_output_configs()
    //    .expect("error while querying configs");
    //let supported_config = supported_configs_range.next()
    //    .expect("no supported config")
    //    .with_sample_rate(SampleRate(sr));
    //let sample_format = supported_config.sample_format();
    let config = StreamConfig {
        channels: channels as u16,
        sample_rate: SampleRate(sr), // Audio device default sample rate is set to 192000
        buffer_size: BufferSize::Default,
    };

    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

    let mut data_interleaved_iter = data_interleaved.into_iter();
    let mut next_value = move || {
        data_interleaved_iter
            .next()
            .expect("cannot get next iter value")
    };

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value)
            },
            err_fn,
        )
        .unwrap();

    fn write_data<T: Sample>(
        output: &mut [T],
        channels: usize,
        next_sample: &mut dyn FnMut() -> f32,
    ) {
        for frame in output.chunks_mut(channels) {
            for sample in frame.iter_mut() {
                let value: T = Sample::from(&next_sample());
                *sample = value
            }
        }
    }

    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5000));
}

#[cfg(test)]
mod test_play {
    use super::*;
    use crate::decode_symphonia;
    use extendr_api::NA_REAL;
    use std::path::Path;

    #[test]
    fn test_play() {
        let fname = "../../test_files/mono.wav";
        let path = Path::new(fname);
        let filetype = Path::extension(path)
            .expect("couldn't extract the file extension")
            .to_str()
            .expect("cannot convert from &OsStr to &str");
        let decoded_arr = decode_symphonia::load(path, false, 0., NA_REAL, filetype);
        let sr = decode_symphonia::get_samplerate(path, filetype);
        println!("{:?}", decoded_arr);
        println!("{:?}", sr);
        play(&decoded_arr.view(), sr);
    }
}
