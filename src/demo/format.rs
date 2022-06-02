use anyhow::Result;
use bincode::{config::Configuration, Decode, Encode};
use std::{
    fs::File,
    io::{BufReader, BufWriter}, path::Path,
};

#[derive(Encode, Decode, Debug)]
pub enum Event {
    // Trigger
    Kick,
    Snare,
    Hat,
    Trigger { id: u8 },

    // Toggle
    Strobe,
    Toggle { id: u8 },

    // CC
    Mod { id: u8, fr: f32 },

    // Meta
    Program { id: u8 },
}

#[derive(Encode, Decode, Debug, Clone, Copy)]
pub struct Metadata {
    pub sample_rate: u32,
    pub peak_rms: f32,
}

#[derive(Encode, Decode, Debug)]
pub struct Data {
    pub rms: f32,
}

#[derive(Encode, Decode)]
pub struct Demo {
    pub meta: Metadata,

    pub audio: Vec<Vec<f32>>,
    pub events: Vec<(f32, Event)>,
    pub data: Vec<(f32, Data)>,
}

impl Demo {
    pub fn save(&self, file: &str) -> Result<()> {
        if Path::new(file).exists() {
            std::fs::remove_file(file)?;
        }

        let file = File::options()
            .create_new(true)
            .write(true)
            .open(file)?;

        let mut write = BufWriter::new(file);
        bincode::encode_into_std_write(&self, &mut write, bincode::config::standard())?;
        Ok(())
    }

    pub fn load(file: &str) -> Result<Self> {
        let mut read = BufReader::new(File::open(file)?);
        Ok(bincode::decode_from_std_read(
            &mut read,
            bincode::config::standard(),
        )?)
    }

    pub fn load_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(bincode::decode_from_slice(bytes, bincode::config::standard())?.0)
    }

    pub fn new(audio: &str, midi: &str) -> Result<Self> {
        println!("Parsing MIDI events...");
        let events = super::midi::parse_events(midi)?;

        println!("Analyzing audio...");
        let (sample_rate, audio) = super::audio::parse(audio)?;
        let data = super::audio::analyze(&audio, sample_rate, 1024);
        let peak_rms = data
            .iter()
            .map(|(_, data)| data.rms)
            .max_by(|a, b| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        println!("Done!");

        Ok(Self {
            meta: Metadata {
                sample_rate,
                peak_rms,
            },

            audio,
            events,
            data,
        })
    }
}
