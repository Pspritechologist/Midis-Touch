use std::sync::{Arc, Mutex};

use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use midir::{Ignore, MidiIO, MidiInput};
use tinyaudio::{run_output_device, OutputDeviceParameters};

use std::error::Error;
use std::io::{stdin, stdout, Write};

fn main() {
    let mut midi_in = MidiInput::new("midis-touch input").expect("Failed to create MIDI input");
    midi_in.ignore(Ignore::None);

    let in_port = select_port(&midi_in, "input").expect("Failed to select input port");

    let in_port_name = midi_in.port_name(&in_port).unwrap_or_default();

    // Setup the audio output.
    let params = OutputDeviceParameters {
        channels_count: 2,
        sample_rate: 44100,
        channel_sample_count: 441,
    };

    // Create the MIDI file sequencer.
    let settings = SynthesizerSettings::new(params.sample_rate as i32);
    let synthesizer = Arc::new(Mutex::new(
        Synthesizer::new(&midis_touch::DEFAULT_FONT, &settings).expect("Failed to initialize synthesizer.")
    ));

    let synth_c = synthesizer.clone();

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        &in_port,
        "midis-touch input",
        move |_, message, _| {
            synth_c.lock().unwrap().process_midi_message(
                1, 
                message[0] as i32, 
                message.get(1).cloned().unwrap_or_default() as i32, 
                message.get(2).cloned().unwrap_or_default() as i32
            );
        },
        (),
    ).expect("Failed to connect to MIDI input port");

    // Buffer for the audio output.
    let mut left: Vec<f32> = vec![0_f32; params.channel_sample_count];
    let mut right: Vec<f32> = vec![0_f32; params.channel_sample_count];
    
    // Output the audio.
    let _dev = run_output_device(params, move |data| {
        synthesizer.lock().unwrap().render(&mut left[..], &mut right[..]);
        for (i, ch) in data.chunks_mut(2).enumerate() {
            // println!("Outputting...");
            ch[0] = left[i];
            ch[1] = right[i];
        }
    }).expect("Failed to output audio.");


    println!(
        "Taking input from '{}' (press enter to exit) ...",
        in_port_name
    );

    let mut input = String::new();
    _ = stdin().read_line(&mut input); // wait for next enter key press
}

fn select_port<T: MidiIO>(midi_io: &T, descr: &str) -> Result<T::Port, Box<dyn Error>> {
    println!("Available {} ports:", descr);
    let midi_ports = midi_io.ports();
    for (i, p) in midi_ports.iter().enumerate() {
        println!("{}: {}", i, midi_io.port_name(p)?);
    }
    print!("Please select {} port: ", descr);
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let port = midi_ports
        .get(input.trim().parse::<usize>()?)
        .ok_or("Invalid port number")?;
    Ok(port.clone())
}
