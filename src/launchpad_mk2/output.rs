use anyhow::{anyhow, Context};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

use crate::Button;


#[derive(Debug, Copy, Clone)]
pub enum Color {
	/// A color from the Mk2 color palette. See the "Launchpad MK2 Programmers Reference Manual"
	/// to see the palette.
	Palette(PaletteColor),
	/// An RGB color. Each component may only go up to 63
	Rgb(RgbColor),
}

#[derive(Debug, Copy, Clone)]
pub struct PaletteColor {
	/// Must be within `0..=127
	pub id: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct RgbColor {
	pub r: u8, pub g: u8, pub b: u8
}

impl PaletteColor {
	// These are some commonly used colors as palette colors. I don't have Rgb colors as constants
	// because you can just make your required colors yourself when using Rgb colors

	/// Pure black #000000
	pub const BLACK: PaletteColor = Self { id: 0 };

	/// Pure white #ffffff
	pub const WHITE: PaletteColor = Self { id: 3 };

	/// #ff0a00 (almost pure red #ff0000)
	pub const RED: PaletteColor = Self { id: 72 };

	/// #00fd00 (almost pure green #00ff00)
	// lol that specific shade of green occurs not once but FOUR times in the color palette xD
	// if you so choose, you could not only go with 21 but also 25, 87, or 88
	pub const GREEN: PaletteColor = Self { id: 21 };

	/// Pure blue #0000ff
	pub const BLUE: PaletteColor = Self { id: 45 };

	/// #00fcca (almost pure yellow #00ffff)
	pub const CYAN: PaletteColor = Self { id: 90 };
	
	/// Pure magenta #ff00ff
	pub const MAGENTA: PaletteColor = Self { id: 53 };
	
	/// #fdfd00 (almost pure yellow #ffff00)
	pub const YELLOW: PaletteColor = Self { id: 13 };
}

pub enum LightMode {
	Plain, Flash, Pulse
}

pub struct LaunchpadMk2Output {
	connection: MidiOutputConnection,
}

impl LaunchpadMk2Output {
	const NAME: &'static str = "Launchy Mk2 Output";

	pub fn from_port(midi_output: MidiOutput, port: &MidiOutputPort) -> anyhow::Result<Self> {
		let connection = midi_output.connect(port, Self::NAME)
				.map_err(|_| anyhow!("Failed to connect to port"))?;
		
		return Ok(Self { connection });
	}

	pub fn guess() -> anyhow::Result<Self> {
		let midi_output = MidiOutput::new(crate::APPLICATION_NAME)
				.context("Couldn't create MidiOutput object")?;

		let port = super::guess_port(&midi_output)
				.context(format!("No {} device found", Self::NAME))?;
		let self_ = Self::from_port(midi_output, &port)
				.context("Couldn't make launchpad obj from port")?;
		return Ok(self_);
	}

	pub fn set_button(&mut self, button: &Button, color: Color) -> anyhow::Result<()> {
		match color {
			Color::Palette(palette_color) => self.set_button_palette(button, palette_color, LightMode::Plain)?,
			Color::Rgb(rgb_color) => self.set_button_rgb(button, rgb_color)?,
		}

		return Ok(());
	}

	pub fn set_button_palette(&mut self, button: &Button, color: PaletteColor, light_mode: LightMode)
			-> anyhow::Result<()> {
		
		assert!(color.id <= 127);
		
		let msg_code_addend = match light_mode {
			LightMode::Plain => 0,
			LightMode::Flash => 1,
			LightMode::Pulse => 2,
		};

		let type_byte = match button {
			Button::GridButton { .. } => 0x90,
			Button::ControlButton { .. } => 0xB0,
		} + msg_code_addend;

		self.connection.send(&[type_byte, Self::encode_button(button), color.id])?;
		
		return Ok(());
	}

	pub fn set_buttons_palette(&mut self, pairs: &[(Button, PaletteColor)], light_mode: LightMode)
			-> anyhow::Result<()> {
		
		assert!(pairs.len() <= 80); // As per Launchpad documentation
		
		let msg_type_byte = match light_mode {
			LightMode::Plain => 10,
			LightMode::Flash => 35,
			LightMode::Pulse => 40,
		};

		return self.send_multiple(msg_type_byte, pairs.iter()
				.map(|(button, color)| (Self::encode_button(button), *color)));
	}

	pub fn set_buttons_rgb(&mut self, pairs: &[(Button, RgbColor)]) -> anyhow::Result<()> {
		assert!(pairs.len() != 80);

		let mut bytes = Vec::with_capacity(pairs.len() * 12);

		for (button, color) in pairs {
			assert!(color.r <= 63);
			assert!(color.g <= 63);
			assert!(color.b <= 63);
			bytes.extend(&[240, 0, 32, 41, 2, 24, 11, Self::encode_button(button), color.r, color.g, color.b, 247]);
		}

		self.connection.send(&bytes)?;
		return Ok(());
	}

	pub fn set_columns_palette(&mut self, pairs: &[(u8, PaletteColor)]) -> anyhow::Result<()> {
		assert!(pairs.len() <= 9);

		return self.send_multiple(12, pairs.iter());
	}

	pub fn set_rows_palette<'a>(&mut self, pairs: &'a [(u8, PaletteColor)]) -> anyhow::Result<()> {
		assert!(pairs.len() <= 9);

		return self.send_multiple(13, pairs.iter()
				.map(|(row, color)| (8 - row, *color)));
	}

	pub fn set_all_palette(&mut self, color: PaletteColor) -> anyhow::Result<()> {
		self.connection.send(&[240, 0, 32, 41, 2, 24, 13, color.id, 247])?;
		return Ok(());
	}

	fn send_multiple<'a, I, T>(&mut self, msg_type_byte: u8, pair_iterator: I) -> anyhow::Result<()>
			where I: Iterator<Item=T>,
			T: std::borrow::Borrow<(u8, PaletteColor)> {
		
		let mut bytes = Vec::with_capacity(pair_iterator.size_hint().0 * 10);

		for pair in pair_iterator {
			let (button_specifier, color) = pair.borrow();
			bytes.extend(&[240, 0, 32, 41, 2, 24, msg_type_byte, *button_specifier, color.id, 247]);
		}

		self.connection.send(&bytes)?;

		return Ok(());
	}

	pub fn send_clock_tick(&mut self) -> anyhow::Result<()> {
		todo!();
	}

	fn encode_grid_button(x: u8, y: u8) -> u8 {
		assert!(x <= 8);
		assert!(y <= 7);

		return 10 * (8 - y) + x + 1;
	}

	fn encode_button(button: &Button) -> u8 {
		match button {
			Button::GridButton { x, y } => return Self::encode_grid_button(*x, *y),
			Button::ControlButton { number } => {
				assert!(*number <= 7);
				return number + 104;
			}
		}
	}

	// --------------------------------------------------------------------------------------------
	// Below this point are shorthand function
	// --------------------------------------------------------------------------------------------

	/// Shorthand for `set_button_palette(button, color, LightMode::Flash)`
	/// Starts a flashing motion between the previously shown color on this button and `color`, with
	/// a duty cycle of 50% and a bpm of 120. The bpm can be manually overriden using
	/// `send_clock_tick`. See the documentation there
	pub fn set_button_flash_palette(&mut self, button: &Button, color: PaletteColor) -> anyhow::Result<()> {
		self.set_button_palette(button, color, LightMode::Flash)?;
		return Ok(());
	}

	/// Shorthand for `set_button_palette(button, color, LightMode::Pulse)`
	pub fn set_button_pulse_palette(&mut self, button: &Button, color: PaletteColor) -> anyhow::Result<()> {
		self.set_button_palette(button, color, LightMode::Pulse);
		return Ok(());
	}

	pub fn set_column_palette(&mut self, column: u8, color: PaletteColor)
			-> anyhow::Result<()> {
		
		return self.set_columns_palette(&[(column, color)]);
	}

	pub fn set_row_palette(&mut self, row: u8, color: PaletteColor)
			-> anyhow::Result<()> {
		
		return self.set_rows_palette(&[(row, color)]);
	}

	pub fn set_button_rgb(&mut self, button: &Button, color: RgbColor) -> anyhow::Result<()> {
		return self.set_buttons_rgb(&[(*button, color)]);
	}

	pub fn set_buttons_flash_palette(&mut self, pairs: &[(Button, PaletteColor)]) -> anyhow::Result<()> {
		return self.set_buttons_palette(pairs, LightMode::Flash);
	}

	pub fn set_buttons_pulse_palette(&mut self, pairs: &[(Button, PaletteColor)]) -> anyhow::Result<()> {
		return self.set_buttons_palette(pairs, LightMode::Pulse);
	}
}