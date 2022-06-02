use anyhow::{Context, Result};
use parking_lot::{Condvar, Mutex};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, StreamConfig};
use rodio::{Decoder, Source};

use rustfft::num_complex::Complex32;
use rustfft::num_traits::Zero as _;
use rustfft::FftPlanner;

use super::{Data, Metadata};

pub struct Stream {
    playing: Arc<AtomicBool>,
}

impl Stream {
    pub fn new(meta: Metadata, mut audio: Vec<Vec<f32>>, start: f32) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .context("no compatible audio device")?;
        let config = StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(meta.sample_rate as u32),
            buffer_size: cpal::BufferSize::Default,
        };

        let playing = Arc::new(AtomicBool::new(false));
        let playing_inner = Arc::clone(&playing);

        thread::spawn(move || {
            let mut it_left = audio.remove(0).into_iter();
            let mut it_right = audio.remove(0).into_iter();

            let skip = (start * meta.sample_rate as f32).round() as u32;
            for _ in 0..skip {
                it_left.next().unwrap_or(0.0);
                it_right.next().unwrap_or(0.0);
            }

            let stream = device
                .build_output_stream(
                    &config.into(),
                    move |output: &mut [f32], _| {
                        if playing_inner.load(Ordering::SeqCst) {
                            for frame in output.chunks_exact_mut(2) {
                                frame[0] = it_left.next().unwrap_or(0.0);
                                frame[1] = it_right.next().unwrap_or(0.0);
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

    pub fn play(&self) {
        self.playing.store(true, Ordering::SeqCst);
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
    let window: Vec<_> = apodize::hanning_iter(fft_size)
        .map(|v| v as f32)
        .collect();
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

        data.push((t, Data {
            rms,
        }));
    }

    data
}
