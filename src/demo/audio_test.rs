use parking_lot::{Condvar, Mutex};
use std::sync::Arc;
use anyhow::{Result, Context};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Host, Device, StreamConfig, SampleRate};


/// Create an audio host and device
fn setup() -> Result<(Host, Device)> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("no compatible audio device")?;

    Ok((host, device))
}

/// Create an output stream and get its buffer size
fn run(device: &Device, config: &StreamConfig) -> Result<usize> {
    let cond = Arc::new(Condvar::new());
    let size = Arc::new(Mutex::new(0usize));

    let cond_inner = Arc::clone(&cond);
    let size_inner = Arc::clone(&size);

    // Run for 4 callbacks
    let mut n = 4;

    let stream = device.build_output_stream(
        config,
        move |output: &mut [f32], _| {
            // Write silence
            for v in output.iter_mut() {
                *v = 0.0;
            }

            let buffer_size = output.len() / 2;
            // println!("buffer size {} = {}", n, buffer_size);

            if n == 1 {
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

#[test]
fn test_0_with_default_config() -> Result<()> {
    let (host, device) = setup()?;

    let default_config = device.default_output_config()?;
    let config: StreamConfig = default_config.clone().into();

    let buffer_size = run(&device, &config)?;

    println!("\n  default_config = {:?}", default_config);
    println!("  config = {:?}", &config);
    println!("  buffer_size = {}", buffer_size);
    println!();
    println!();

    Ok(())
}

#[test]
fn test_1_with_default_sample_rate() -> Result<()> {
    let (host, device) = setup()?;

    let default_config = device.default_output_config()?;
    let config = StreamConfig {
        channels: default_config.channels(),
        sample_rate: default_config.sample_rate(),
        buffer_size: BufferSize::Default,
    };

    let buffer_size = run(&device, &config)?;

    println!("\n  default_config = {:?}", default_config);
    println!("  config = {:?}", config);
    println!("  buffer_size = {}", buffer_size);
    println!();
    println!();

    Ok(())
}

#[test]
fn test_2_with_alt_sample_rate() -> Result<()> {
    let (host, device) = setup()?;

    let default_config = device.default_output_config()?;
    let config = StreamConfig {
        channels: default_config.channels(),
        sample_rate: if default_config.sample_rate().0 == 44100 {
            SampleRate(48000)
        } else {
            SampleRate(44100)
        },
        buffer_size: BufferSize::Default,
    };

    let buffer_size = run(&device, &config)?;

    println!("\n  default_config = {:?}", default_config);
    println!("  config = {:?}", config);
    println!("  buffer_size = {}", buffer_size);
    println!();
    println!();

    Ok(())
}

#[test]
fn test_3_with_custom_sample_rate() -> Result<()> {
    let (host, device) = setup()?;

    let default_config = device.default_output_config()?;
    let config = StreamConfig {
        channels: default_config.channels(),
        sample_rate: SampleRate(192000),
        buffer_size: BufferSize::Default,
    };

    let buffer_size = run(&device, &config)?;

    println!("\n  default_config = {:?}", default_config);
    println!("  config = {:?}", config);
    println!("  buffer_size = {}", buffer_size);
    println!();
    println!();

    Ok(())
}

#[test]
fn test_4_with_low_buffer_size() -> Result<()> {
    let (host, device) = setup()?;

    let default_config = device.default_output_config()?;
    let config = StreamConfig {
        channels: default_config.channels(),
        sample_rate: default_config.sample_rate(),
        buffer_size: BufferSize::Fixed(64),
    };

    let buffer_size = run(&device, &config)?;

    println!("\n  default_config = {:?}", default_config);
    println!("  config = {:?}", config);
    println!("  buffer_size = {}", buffer_size);
    println!();
    println!();

    Ok(())
}

#[test]
fn test_5_with_med_buffer_size() -> Result<()> {
    let (host, device) = setup()?;

    let default_config = device.default_output_config()?;
    let config = StreamConfig {
        channels: default_config.channels(),
        sample_rate: default_config.sample_rate(),
        buffer_size: BufferSize::Fixed(1024),
    };

    let buffer_size = run(&device, &config)?;

    println!("\n  default_config = {:?}", default_config);
    println!("  config = {:?}", config);
    println!("  buffer_size = {}", buffer_size);
    println!();
    println!();

    Ok(())
}

#[test]
fn test_6_with_high_buffer_size() -> Result<()> {
    let (host, device) = setup()?;

    let default_config = device.default_output_config()?;
    let config = StreamConfig {
        channels: default_config.channels(),
        sample_rate: default_config.sample_rate(),
        buffer_size: BufferSize::Fixed(4096),
    };

    let buffer_size = run(&device, &config)?;

    println!("\n  default_config = {:?}", default_config);
    println!("  config = {:?}", config);
    println!("  buffer_size = {}", buffer_size);
    println!();
    println!();

    Ok(())
}

#[test]
fn test_7_with_default_channels() -> Result<()> {
    let (host, device) = setup()?;

    let default_config = device.default_output_config()?;
    let config = StreamConfig {
        channels: default_config.channels(),
        sample_rate: default_config.sample_rate(),
        buffer_size: BufferSize::Default,
    };

    let buffer_size = run(&device, &config)?;

    println!("\n  default_config = {:?}", default_config);
    println!("  config = {:?}", config);
    println!("  buffer_size = {}", buffer_size);
    println!();
    println!();

    Ok(())
}

fn test_8_with_2_channels() -> Result<()> {
    let (host, device) = setup()?;

    let default_config = device.default_output_config()?;
    let config = StreamConfig {
        channels: 2,
        sample_rate: default_config.sample_rate(),
        buffer_size: BufferSize::Default,
    };

    let buffer_size = run(&device, &config)?;

    println!("\n  default_config = {:?}", default_config);
    println!("  config = {:?}", config);
    println!("  buffer_size = {}", buffer_size);
    println!();
    println!();

    Ok(())
}