#![feature(duration_constructors)]

use std::{cmp::min, fmt::{Debug, Display}, fs::File, sync::{Arc, Mutex}, time::Duration};

use midis_touch::soundfonts::SoundFontUtils;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use tinyaudio::{run_output_device, OutputDeviceParameters};

fn main() -> Result<(), AppError> {
    // Get midi file as first argument.
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: midis-touch <path to midi>");
        return Ok(());
    }
    let midi_file_path = &args[1];

    // Setup the audio output.
    let params = OutputDeviceParameters {
        channels_count: 2,
        sample_rate: 44100,
        channel_sample_count: 1024,
    };

    // Buffer for the audio output.
    let mut left: Vec<f32> = vec![0_f32; params.channel_sample_count];
    let mut right: Vec<f32> = vec![0_f32; params.channel_sample_count];

    // Load the MIDI file.
    let mut midi_file = match File::open(midi_file_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open MIDI file '{midi_file_path}': {e}").into())
    };

    let midi_file = Arc::new(match MidiFile::new(&mut midi_file) {
        Ok(midi_file) => midi_file,
        Err(e) => return Err(format!("Failed to load MIDI data: {e}").into())
    });

    // Create the MIDI file sequencer.
    let settings = SynthesizerSettings::new(params.sample_rate as i32);
    let synthesizer = Synthesizer::new(&SoundFont::default(), &settings)?;
    let mut sequencer = MidiFileSequencer::new(synthesizer);

    // Play the MIDI file.
    sequencer.play(&midi_file, false);

    let sequencer = Arc::new(Mutex::new(sequencer));
    let seq_ref = sequencer.clone();

    println!("Playing MIDI file...");
    // Start the audio output.
    // You need to keep a reference to the device to keep the audio output running.
    let _device = match run_output_device(params, {
        move |data| {
            seq_ref.lock().unwrap().render(&mut left[..], &mut right[..]);
            for (i, val) in data.chunks_mut(2).enumerate() {
                val[0] = left[i];
                val[1] = right[i];
            }
        }
    }) {
        Ok(device) => device,
        Err(e) => return Err(format!("Failed to start audio output: {e}").into())
    };

    println!("Waiting for MIDI file to finish...");
    while !sequencer.lock().unwrap().end_of_sequence() {
        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

struct AppError(Box<dyn Display>);

impl Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}
impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl From<String> for AppError {
    fn from(e: String) -> Self {
        Self(Box::new(e))
    }
}
impl From<rustysynth::SynthesizerError> for AppError {
    fn from(e: rustysynth::SynthesizerError) -> Self {
        Self(Box::new(e))
    }
}
