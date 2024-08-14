use std::{fs, sync::{Arc, Mutex}};

use eframe::egui::{self, containers, widgets};
use midis_touch::soundfonts::SoundFontUtils;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};

pub struct App {
	// pub state: Arc<Mutex<AppState>>,
	fonts: Vec<Arc<SoundFont>>,
	selected_font: usize,
	pub seq: Arc<Mutex<MidiFileSequencer>>,
	// synth_settings: SynthesizerSettings,
}

impl App {
	pub fn new(synth: Arc<Mutex<MidiFileSequencer>>) -> Self {
		// let setts = SynthesizerSettings::new(sample_rate);
		
		Self {
			// state: Arc::new(Mutex::new(AppState::default())),
			fonts: vec![ midis_touch::DEFAULT_FONT.clone() ],
			selected_font: 0,
			seq: synth,
			// setts,
		}
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		let mut seq = self.seq.lock().unwrap();
		// let mut state = self.state.lock().unwrap();

		egui::TopBottomPanel::top("uwu").show(ctx, |ui| {
			ui.horizontal(|ui| {
				if egui::ComboBox::from_label("uwu")
					.show_index(
						ui,
						&mut self.selected_font,
						self.fonts.len(),
						|i| get_font_name(&self.fonts[i])
					)
					.changed() {
						let sample = seq.get_synthesizer().get_sample_rate();
						seq.set_synthesizer(Synthesizer::new(&self.fonts[self.selected_font], &SynthesizerSettings::new(sample)).unwrap());
					}
			});
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading("Synthesizer");

			ui.horizontal(|ui| {
				ui.label("Volume:");

				ui.add(egui::Slider::from_get_set(0.0..=5.0, |value| match value {
					Some(value) => {
						seq.get_mut_synthesizer().set_master_volume(value as f32);
						value
					},
					None => seq.get_synthesizer().get_master_volume() as f64
				})
					.text("Volume"));
			});

			ui.horizontal(|ui| {
				ui.label("Scrub:");
				let value = seq.get_position() / seq.get_midi_file().map_or(0., MidiFile::get_length);
				let scrub = ui.add(egui::ProgressBar::new(value as f32));

				if scrub.interact(egui::Sense::drag()).dragged() {
					seq.offset_position(scrub.drag_motion().x as f64, false);
				} else if scrub.drag_stopped() {
					seq.offset_position(0., true);
				}
				
			});

			// if let Some(payload) = <&str, _>(egui::Frame { inner_margin: egui::Margin::same(3.), outer_margin: egui::Margin::same(4.), ..Default::default() }, |ui| {
			// 	ui.label("Drop MIDI file here");
			// 	return 4;
			// }).1 {
			// 	self.fonts.push(SoundFont::from_path(&payload).unwrap());
			// };
		});

		ctx.input(|input| {
			for file in input.raw.dropped_files.iter() {
				if let Ok(Ok(sf)) = SoundFont::try_from_path(file.path.to_owned().unwrap_or("".into())) {
					self.fonts.push(sf);
				};
			}
		});

		ctx.request_repaint();
	}
}

fn get_font_name(font: &SoundFont) -> &str {
	if !font.get_info().get_rom_name().is_empty() {
		font.get_info().get_rom_name()	
	} else if !font.get_info().get_bank_name().is_empty() {
		font.get_info().get_bank_name()
	} else if !font.get_info().get_target_product().is_empty() {
		font.get_info().get_target_product()
	} else if !font.get_info().get_target_sound_engine().is_empty() {
		font.get_info().get_target_sound_engine()
	} else if font.get_info().get_comments().is_empty() {
		font.get_info().get_comments()
	} else if font.get_info().get_author().is_empty() {
		font.get_info().get_author()
	} else {
		"Unknown"
	}
}

// #[derive(Default)]
// pub struct AppState {
// 	pub playing: bool,
// }
