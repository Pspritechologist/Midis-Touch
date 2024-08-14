use std::sync::{Arc, Mutex};

use eframe::{App, NativeOptions};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use midir::{Ignore, MidiIO, MidiInput, MidiInputPort};
use tinyaudio::{run_output_device, OutputDeviceParameters};

use std::error::Error;
use std::io::{stdin, stdout, Write};

fn main() {
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

    let mut midi_in = MidiInput::new("midis-touch input").expect("Failed to create MIDI input");
    midi_in.ignore(Ignore::None);

    #[cfg(not(feature = "gui"))] let _conn_in;
    #[cfg(not(feature = "gui"))] let in_port_name;

    #[cfg(not(feature = "gui"))]
    {
        let in_port = select_port(&midi_in, "input").expect("Failed to select input port");
    
        in_port_name = midi_in.port_name(&in_port).unwrap_or_default();

        // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
        _conn_in = midi_in.connect(
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
    }

    // Buffer for the audio output.
    let mut left: Vec<f32> = vec![0_f32; params.channel_sample_count];
    let mut right: Vec<f32> = vec![0_f32; params.channel_sample_count];
    
    // Output the audio.
    let _dev = run_output_device(params, move |data| {
        synthesizer.lock().unwrap().render(&mut left[..], &mut right[..]);
        for (i, ch) in data.chunks_mut(2).enumerate() {
            ch[0] = left[i];
            ch[1] = right[i];
        }
    }).expect("Failed to output audio.");

    #[cfg(feature = "gui")]
    {
        let mut app = gui::AppWindow::new(midi_in, synth_c);

        eframe::run_simple_native("Midis SW Player", NativeOptions::default(), move |ctx, frame| app.update(ctx, frame)).expect("GUI Error");
    }

    #[cfg(not(feature = "gui"))]
    {
        println!(
            "Taking input from '{}' (press enter to exit) ...",
            in_port_name
        );

        let mut input = String::new();
        _ = stdin().read_line(&mut input); // wait for next enter key press
    }
}

#[cfg(not(feature = "gui"))]
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

#[cfg(feature = "gui")]
mod gui {
    use super::*;
    use eframe::egui::{self, CentralPanel, ComboBox};
    use midir::MidiInputConnection;

    pub struct AppWindow {
        pub midi_input: MidiInput,
        pub synth: Arc<Mutex<Synthesizer>>,
        pub connection: Option<MidiInputConnection<()>>,

        combo: bool,
        selected: usize,
    }

    impl AppWindow {
        pub fn new(midi_input: MidiInput, synth: Arc<Mutex<Synthesizer>>) -> Self {
            Self {
                midi_input,
                synth,
                connection: None,
                combo: false,
                selected: 0,
            }
        }

        fn make_connection(&mut self, port: &MidiInputPort) {
            let synth_c = self.synth.clone();
            self.connection = Some(MidiInput::new("temp").unwrap().connect(
                &port,
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
            ).expect("Failed to connect to MIDI input port"));
        }
    }

    impl eframe::App for AppWindow {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Midi Player :)");
                    ui.checkbox(&mut self.combo, "Dropdown");
                });

                if self.combo {
                    if ComboBox::from_label("Select input").show_index(ui, &mut self.selected, self.midi_input.port_count(), |i| {
                        self.midi_input.port_name(&self.midi_input.ports()[i]).unwrap_or_default()
                    }).changed() {
                        self.make_connection(&self.midi_input.ports()[self.selected]);
                    }
                } else {
                    ui.menu_button("Change input", |ui| {
                        if ui.button("Close").clicked() {
                            ui.close_menu();
                        }
                        ui.vertical_centered(move|ui| {
                            for port in self.midi_input.ports().iter() {
                                if ui.button(self.midi_input.port_name(port).unwrap_or("ERROR".into())).clicked() {
                                    self.make_connection(port);
                                }
                            }
                        });
                    });
                }
            });
        }
    }
}
