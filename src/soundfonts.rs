use std::sync::{Arc, LazyLock};

use rustysynth::{SoundFont, SoundFontError};

// We pick a default font based on features.
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

// We also allow building with a custom soundfont!
#[cfg(feature = "custom_font")]
const SF2_BYTES: &[u8] = include_bytes!("../custom_font.sf2");

/// Whichever soundfont is 'default' in this build.  
/// Usually ranges from 'cute and retro' at less than 100kb to
/// 'full and rich' at 7mb depending on the feature flags.  
/// Also accepts a custom font upon building.
#[allow(const_item_mutation)]
pub static DEFAULT_FONT: LazyLock<Arc<SoundFont>> = LazyLock::new(||
	Arc::new(SoundFont::new(&mut SF2_BYTES).expect("Failed to initialize soundfont."))
);

/// Some basic utils for loading soundfonts.
pub trait SoundFontUtils {
	/// Returns a new `Arc` to the [Default Soundfont](DEFAULT_FONT).
	fn default() -> Arc<SoundFont>;
	/// Tries to load a soundfont from a given file path.
	fn try_from_path(path: &str) -> Result<Result<Arc<SoundFont>, std::io::Error>, SoundFontError>;
	/// Tries to load a soundfont from a given byte slice.
	fn try_from_bytes(bytes: &[u8]) -> Result<Arc<SoundFont>, SoundFontError>;

	/// Loads a soundfont from a given file path.  
	/// Panics if the soundfont cannot be loaded.
	/// Does not panic if the file is not found.
	fn from_path(path: &str) -> Result<Arc<SoundFont>, std::io::Error> {
		Self::try_from_path(path).expect("Failed to initialize soundfont")
	}

	/// Loads a soundfont from a given byte slice.  
	/// Panics if the soundfont cannot be loaded.
	fn from_bytes(bytes: &[u8]) -> Arc<SoundFont> {
		Self::try_from_bytes(bytes).expect("Failed to initialize soundfont")
	}
}

impl SoundFontUtils for SoundFont {
	fn try_from_path(path: &str) -> Result<Result<Arc<SoundFont>, std::io::Error>, SoundFontError> {
		Ok(Ok(
			Arc::new(SoundFont::new(&mut std::fs::read(path)?.as_slice())?)
		))
	}

	fn try_from_bytes(mut bytes: &[u8]) -> Result<Arc<SoundFont>, SoundFontError> {
		Ok(Arc::new(SoundFont::new(&mut bytes)?))
	}

	fn default() -> Arc<SoundFont> {
		DEFAULT_FONT.clone()
	}
}
