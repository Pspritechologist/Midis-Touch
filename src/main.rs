#![feature(duration_constructors)]

use std::{fs::File, sync::Arc, time::Duration};

use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use tinyaudio::{run_output_device, OutputDeviceParameters};

#[cfg(feature = "7mb_font")]
const SF2_BYTES: &[u8] = include_bytes!("../soundfonts/7mb_font.sf2");
#[cfg(feature = "1mb_font")]
const SF2_BYTES: &[u8] = include_bytes!("../soundfonts/1mb_font.sf2");
#[cfg(feature = "200kb_font")]
const SF2_BYTES: &[u8] = include_bytes!("../soundfonts/200kb_font.sf2");
#[cfg(feature = "100kb_font")]
const SF2_BYTES: &[u8] = include_bytes!("../soundfonts/100kb_font.sf2");
#[cfg(feature = "60kb_font")]
const SF2_BYTES: &[u8] = include_bytes!("../soundfonts/60kb_font.sf2");

fn main() {
    // Get midi file as first argument.
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: midis-touch <path to midi>");
        return;
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

    // Load the SoundFont.
    #[allow(const_item_mutation)]
    let sound_font = Arc::new(SoundFont::new(&mut SF2_BYTES).expect("Failed to initialize soundfont."));

    // Load the MIDI file.
    let mut midi_file = match File::open(midi_file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to load MIDI file: {e}");
            return;
        }
    };

    let midi_file = Arc::new(match MidiFile::new(&mut midi_file) {
        Ok(midi_file) => midi_file,
        Err(e) => {
            eprintln!("Failed to load MIDI file: {e}");
            return;
        },
    });

    // Create the MIDI file sequencer.
    let settings = SynthesizerSettings::new(params.sample_rate as i32);
    let synthesizer = Synthesizer::new(&sound_font, &settings).expect("Failed to initialize synthesizer.");
    let mut sequencer = MidiFileSequencer::new(synthesizer);

    // Play the MIDI file.
    sequencer.play(&midi_file, false);

    println!("Playing MIDI file...");
    // Start the audio output.
    // You need to keep a reference to the device to keep the audio output running.
    let _device = match run_output_device(params, {
        move |data| {
            sequencer.render(&mut left[..], &mut right[..]);
            for (i, val) in data.chunks_mut(2).enumerate() {
                val[0] = left[i];
                val[1] = right[i];
            }
        }
    }) {
        Ok(device) => device,
        Err(e) => {
            eprintln!("Failed to start audio output: {e}");
            return;
        }
    };

    println!("Waiting for MIDI file to finish...");
    // Wait for the duration of the MIDI file.
    std::thread::sleep(Duration::from_secs_f64(midi_file.get_length()));
}
