use anyhow::{Context, Result};
use parking_lot::{Condvar, Mutex};
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::slice::Iter;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Device, StreamConfig};
use rodio::{Decoder, Source};
use rubato::{FftFixedOut, Resampler};

use rustfft::num_complex::Complex32;
use rustfft::num_traits::Zero as _;
use rustfft::FftPlanner;

use super::{Data, Metadata};

pub struct Stream {
    playing: Arc<AtomicBool>,
}

impl Stream {
    /// Start playing an audio stream
    pub fn new(meta: Metadata, mut audio: Vec<Vec<f32>>, start: f32) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .context("no compatible audio device")?;
        let default_config = device
            .default_output_config()
            .context("no compatible default audio output stream")?;

        let config = StreamConfig {
            channels: 2,
            sample_rate: default_config.sample_rate(),
            buffer_size: BufferSize::Default,
        };

        log::info!(
            "using device: {:?}",
            device.name().unwrap_or("unknown".into())
        );
        log::info!("using output config: {:?}", default_config);
        assert!(default_config.channels() == 2);

        let sample_rate_in = meta.sample_rate as usize;
        let sample_rate_out = default_config.sample_rate().0 as usize;
        let buffer_size = Self::preflight(&device, &config)
            .context("stream preflight failed to read buffer size")?;

        let mut resample = Resample::new(sample_rate_in, sample_rate_out, buffer_size)?;

        let playing = Arc::new(AtomicBool::new(false));
        let playing_inner = Arc::clone(&playing);

        thread::spawn(move || {
            // Create iterators over the left and right audio channels
            let mut in_left = audio.remove(0).into_iter();
            let mut in_right = audio.remove(0).into_iter();

            // If `start` is set, skip over the correct number of samples
            let skip = (start * meta.sample_rate as f32).round() as u32;
            for _ in 0..skip {
                in_left.next().unwrap_or(0.0);
                in_right.next().unwrap_or(0.0);
            }

            let stream = device
                .build_output_stream(
                    &default_config.into(),
                    move |output: &mut [f32], _| {
                        if !playing_inner.load(Ordering::SeqCst) || output.len() / 2 != buffer_size {
                            // If we aren't playing yet or the buffer_size doesn't match, write silence
                            for v in output.iter_mut() {
                                *v = 0.0;
                            }
                        } else {
                            // Otherwise write the (possibly) resampled output
                            let (mut out_left, mut out_right) = resample.process(&mut in_left, &mut in_right);
                            for frame in output.chunks_exact_mut(2) {
                                frame[0] = *out_left.next().unwrap_or(&0.0);
                                frame[1] = *out_right.next().unwrap_or(&0.0);
                            }
                        }
                    },
                    |err| log::error!("{:?}", err),
                )
                .expect("failed to create audio stream");

            stream.play().unwrap();

            loop {
                thread::park();
            }
        });

        Ok(Self { playing })
    }

    /// Test-run a SupportedStreamConfig, returning the actual buffer size.
    ///
    /// This is needed since SupportedStreamConfig only gives us a range
    /// of supported buffer sizes, but we need a concrete size to allocate
    /// buffers for the resampler if the sample rates are different.
    pub fn preflight(device: &Device, config: &StreamConfig) -> Result<usize> {
        let cond = Arc::new(Condvar::new());
        let size = Arc::new(Mutex::new(0usize));

        let cond_inner = Arc::clone(&cond);
        let size_inner = Arc::clone(&size);

        // In some cases, the initial callback requests more samples than
        // the "actual" buffer size.
        //
        // To avoid getting the buffer size wrong, we just wait several
        // callbacks and then take the latest buffer size.
        let mut n = 4;

        let stream = device.build_output_stream(
            config,
            move |output: &mut [f32], _| {
                // Write silence
                for v in output.iter_mut() {
                    *v = 0.0;
                }

                let buffer_size = output.len() / 2;
                log::debug!("preflight: buffer size {} = {}", n, buffer_size);

                if n == 1 {

                    // Write output and notify condition variable
                    // *technically* we shouldn't be locking in the audio callback...
                    *size_inner.lock() = buffer_size;
                    cond_inner.notify_one();
                }
                n -= 1;
            },
            |err| log::error!("{:?}", err),
        )?;
        stream.play()?;

        // Wait for the stream thread to signal the buffer size has been read
        let mut value = size.lock();
        while *value == 0 {
            cond.wait(&mut value);
        }

        Ok(*value)
    }

    pub fn play(&self) {
        self.playing.store(true, Ordering::SeqCst);
    }
}

pub struct Resample {
    rate_in: usize,
    rate_out: usize,
    buffer_size: usize,

    resampler: Option<FftFixedOut<f32>>,
    input: Vec<Vec<f32>>,
    output: Vec<Vec<f32>>,
}



impl Resample {
    pub fn new(rate_in: usize, rate_out: usize, buffer_size: usize) -> Result<Self> {
        if rate_in == rate_out {
            Ok(Self {
                rate_in,
                rate_out,
                buffer_size,

                resampler: None,
                input: vec![],
                output: vec![],
            })
        } else {
            log::info!(
                "resampling from {}kHz -> {}kHz",
                rate_in,
                rate_out
            );

            let resampler = FftFixedOut::new(rate_in, rate_out, buffer_size, 2, 2)
                .context("failed to create FFT resampler")?;

            let input = resampler.input_buffer_allocate();
            let output = resampler.output_buffer_allocate();

            Ok(Self {
                rate_in,
                rate_out,
                buffer_size,

                resampler: Some(resampler),
                input,
                output
            })
        }
    }

    pub fn process<L, R>(&mut self, in_left: &mut L, in_right: &mut R) -> (Iter<'_, f32>, Iter<'_, f32>)
    where
        L: Iterator<Item = f32>,
        R: Iterator<Item = f32>,
    {
        if let Some(resampler) = self.resampler.as_mut() {
            self.input[0].resize(resampler.input_frames_next(), 0.0);
            self.input[1].resize(resampler.input_frames_next(), 0.0);
            for l in self.input[0].iter_mut() {
                *l = in_left.next().unwrap_or(0.0);
            }
            for r in self.input[1].iter_mut() {
                *r = in_right.next().unwrap_or(0.0);
            }

            resampler.process_into_buffer(&self.input, &mut self.output, None).unwrap();
        } else {
            for l in self.output[0].iter_mut() {
                *l = in_left.next().unwrap_or(0.0);
            }
            for r in self.output[1].iter_mut() {
                *r = in_right.next().unwrap_or(0.0);
            }
        }

        (self.output[0].iter(), self.output[1].iter())
    }
}



pub fn parse(file: &str) -> Result<(u32, Vec<Vec<f32>>)> {
    let file = BufReader::new(File::open(file)?);
    let decoder = Decoder::new(file)?;

    let sample_rate = decoder.sample_rate();
    assert!(decoder.channels() == 2);

    let interleaved: Vec<f32> = decoder.convert_samples::<f32>().collect();

    let mut left = Vec::new();
    let mut right = Vec::new();
    for p in interleaved.chunks_exact(2) {
        left.push(p[0]);
        right.push(p[1]);
    }

    Ok((sample_rate, vec![left, right]))
}

pub fn analyze(audio: &Vec<Vec<f32>>, sample_rate: u32, fft_size: usize) -> Vec<(f32, Data)> {
    // Set up buffers for the complex FFT I/O, and result
    let mut complex = vec![Complex32::zero(); fft_size * 2];
    let zeros = complex.clone();
    let mut result = vec![0.0; fft_size];

    // Set up the FFT
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size * 2);
    let mut scratch = vec![Complex32::zero(); fft.get_inplace_scratch_len()];

    // Set up the window and calculate the factor we need to scale the FFT result by
    let window: Vec<_> = apodize::hanning_iter(fft_size).map(|v| v as f32).collect();
    let window_factor = window.iter().map(|x| *x as f32).sum::<f32>();

    let mut max = 0.0;
    let mono: Vec<f32> = audio[0]
        .iter()
        .zip(audio[1].iter())
        .map(|(l, r)| (*l + *r) / 2.0)
        .collect();

    let mut data = Vec::new();

    for (i, frame) in mono.chunks(fft_size).enumerate() {
        frame
            .iter()
            .zip(window.iter())
            .zip(complex.iter_mut())
            .for_each(|((sample, w), c)| c.re = sample * *w);

        fft.process_with_scratch(&mut complex, &mut scratch);

        // Copy the abs of each complex result scaled by the window factor into the result buffer
        complex
            .iter()
            .take(fft_size)
            .zip(result.iter_mut())
            .for_each(|(c, v)| {
                *v = c.norm_sqr().sqrt() / window_factor;
            });

        // Zero the complex buffer to clear out the in-place modifications.
        complex.copy_from_slice(&zeros);

        let sum: f32 = result.iter().map(|s| s.abs().powi(2)).sum();
        let rms = (sum / result.len() as f32).sqrt();
        if rms > max {
            max = rms;
        }

        let t = (i * fft_size) as f32 / sample_rate as f32;

        data.push((t, Data { rms }));
    }

    data
}
