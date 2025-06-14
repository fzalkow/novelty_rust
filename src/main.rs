use std::f32::consts::E;
use std::io::Write;
use std::path::Path;

use clap::Parser;
use hann_rs::get_hann_window;
use ndarray::{Array, Array1, s, concatenate, Axis};
use ndarray_conv::{ConvExt, ConvMode, PaddingMode};
use ndarray_stats::QuantileExt;
use wavers::{Wav, Samples, read};

/// Reads a mono WAV file from the given path and returns the audio samples as a 1D array,
/// along with the sampling rate.
///
/// # Errors
/// Returns an error if the file can't be read or if it is not mono.
fn audio_path_to_array(path: &str) -> anyhow::Result<(Array1<f32>, u32)> {
    let reader: Wav<i16> = Wav::from_path(path)?;

    if reader.n_channels() != 1 {
        anyhow::bail!("Can only handle mono files currently. Please convert input audio file to mono.");
    }

    let (samples, sample_rate): (Samples<i16>, i32) = read::<i16, _>(path)?;
    let samples: Vec<f32> = samples.convert().to_vec();
    let audio_array = Array::from_vec(samples);

    Ok((audio_array, sample_rate as u32))
}

/// Computes an energy-based novelty function over the input audio signal.
///
/// This function calculates the short-time energy using a Hann window, applies optional
/// logarithmic compression, computes the positive energy difference over time, and
/// normalizes the result if specified.
///
/// # Arguments
/// - `audio_array`: 1D array of mono audio samples
/// - `fs`: Sampling rate of the audio
/// - `window_length`: Size of the analysis window
/// - `hop_length`: Step size between successive frames
/// - `gamma`: Compression parameter for logarithmic scaling
/// - `norm`: Whether to normalize the output between 0 and 1
///
/// # Returns
/// - A tuple of the novelty function and its effective sampling rate
///
/// # Errors
/// Returns an error if convolution or array operations fail.
fn novelty_energy(audio_array: Array1<f32>, fs: u32, window_length: u32, hop_length: u32 , gamma: f32, norm: bool) -> anyhow::Result<(Array1<f32>, f32)> {
    // get window function
    let hann_window = get_hann_window(window_length as usize).expect("Failed to get the Hann window");
    let hann_window_array = Array::from_vec(hann_window);

    // Compute the feature sampling rate
    let fs_feature = (fs as f32) / (hop_length as f32);

    // Compute local energy with squared window and signal
    let energy_local = audio_array.powf(2.0).conv(&hann_window_array.powf(2.0), ConvMode::Same, PaddingMode::Zeros)?;
    let mut energy_local_subsample = energy_local.slice_move(s![..;hop_length]);

    // Apply logarithmic compression if gamma > 0
    if gamma != 0.0 {
        energy_local_subsample.mapv_inplace(|v| (1.0 + gamma * v).log(E));
    }

    // Compute the difference of consecutive energy values
    let mut energy_local_diff = &energy_local_subsample.slice(s![1..]) - &energy_local_subsample.slice(s![..-1]);

    // Pad with a trailing zero to maintain the original length
    energy_local_diff = concatenate(Axis(0), &[energy_local_diff.view(), Array::zeros(1).view()])?;

    // Apply half-wave rectification (set negative values to zero)
    let mut novelty_energy = energy_local_diff;
    novelty_energy.mapv_inplace(|v| if v < 0.0 { 0.0 } else { v });

    // Normalize if requested
    if norm {
        let max_value = *novelty_energy.max()?;
        if max_value > 0.0 {
            novelty_energy.mapv_inplace(|v| v / max_value);
        }
    }

    Ok((novelty_energy, fs_feature))
}

/// Writes a CSV file containing time vs. novelty function values.
///
/// # Arguments
/// - `path`: Output file path
/// - `novelty_energy`: 1D array of novelty values
/// - `fs_feature`: Sampling rate of the novelty function
/// - `fs`: Original sampling rate of the audio
///
/// # Errors
/// Returns an error if writing to the file fails.
fn write_csv(path: &str, novelty_energy: Array1<f32>, fs_feature: f32, fs: u32) -> anyhow::Result<()> {
    // Compute the time vector corresponding to each novelty value
    let time = Array::range(0.0, novelty_energy.len() as f32, 1.0) * fs_feature / (fs as f32);

    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path)?;

    // Write header
    writeln!(file, "time,novelty").expect("Could not write to file!");

    // Write time and novelty values
    for (cur_time, cur_novelty) in time.iter().zip(novelty_energy.iter()) {
        writeln!(file, "{:.05},{:.05}", cur_time, cur_novelty).expect("Could not write to file!");
    }

    Ok(())
}

/// Struct to represent and parse command-line arguments.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the input mono audio file (WAV)
    #[arg()]
    path_in: String,

    /// Path to the output CSV file
    #[arg()]
    path_out: String,

    /// Window length in samples (default: 1024)
    #[arg(long, default_value_t = 1024)]
    window_length: u32,

    /// Hop length in samples (default: 256)
    #[arg(long, default_value_t = 256)]
    hop_length: u32,

    /// Logarithmic compression parameter gamma (default: 10.0)
    #[arg(long, default_value_t = 10.0)]
    gamma: f32,

    /// Whether to normalize the novelty function (default: true)
    #[arg(long, default_value_t = true)]
    norm: bool,
}

impl Cli {
    /// Validates that the output file does not already exist.
    fn validate(&self) -> anyhow::Result<()> {
        if Path::new(&self.path_out).exists() {
            anyhow::bail!("Output path must not already exist!");
        }
        Ok(())
    }
}

/// Entry point of the application. Parses arguments, computes the novelty function,
/// and writes the results to a CSV file.
///
/// # Errors
/// Returns an error if any step in the pipeline fails.
fn main() -> anyhow::Result<()> {
    // parse command line arguments
    let args = Cli::parse();
    args.validate()?;

    // get audio file
    let (audio_array, fs) = audio_path_to_array(&args.path_in)?;

    // compute novelty function
    let (novelty_energy, fs_feature) = novelty_energy(audio_array, fs, args.window_length, args.hop_length, args.gamma, args.norm)?;

    // write csv result
    write_csv(&args.path_out, novelty_energy, fs_feature, fs)?;

    Ok(())
}
