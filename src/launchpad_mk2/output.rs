use anyhow::{anyhow, Context};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

use crate::Button;


/// The Launchpad Mk2 has two different ways to represent color. You can either use one of the 128
/// built-in palette colors, or you can create a custom color with custom rgb components.
/// Why would you choose the palette colors when you can just create your required colors yourself?
/// Well some operations on the Mk2 only support palette colors. Besides, sending palette color midi
/// messages is simply faster. Therefore you should aim to use the palette colors when possible.

/// A color from the Mk2 color palette. See the "Launchpad MK2 Programmers Reference Manual"
/// to see the palette, or [see here](http://launchpaddr.com/mk2palette/)
///
/// The `id` field must be 127 or lower
#[derive(Debug, Copy, Clone)]
pub struct PaletteColor {
	pub id: u8,
}

impl PaletteColor {
	pub fn is_valid(&self) -> bool {
		return self.id <= 127;
	}

	pub fn new(id: u8) -> Self {
		let self_ = Self { id };
		assert!(self_.is_valid());
		return self_;
	}
}

#[derive(Debug, Copy, Clone)]
/// An RGB color. Each component may only go up to 63
pub struct RgbColor {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

impl RgbColor {
	/// Check whether the rgb color is valid - each component may only go up to 63.
	pub fn is_valid(&self) -> bool {
		return self.r <= 63 && self.g <= 63 && self.b <= 63;
	}

	/// Create a new RgbColor from the individual component values
	pub fn new(r: u8, g: u8, b: u8) -> Self {
		let self_ = Self { r, g, b };
		assert!(self_.is_valid());
		return self_;
	}
}

impl PaletteColor {
	// These are some commonly used colors as palette colors. I don't have Rgb colors as constants
	// because in the case of rgb colors you can just make your required colors yourself

	pub const BLACK: PaletteColor = Self { id: 0 };
	pub const WHITE: PaletteColor = Self { id: 3 };
	pub const RED: PaletteColor = Self { id: 72 }; // #ff0a00 (almost pure red #ff0000)
	// lol this specific shade of green occurs not once but FOUR times in the color palette xD
	// if you so choose, you could not only go with 21 but also 25, 87, or 88
	pub const GREEN: PaletteColor = Self { id: 21 }; // #00fd00 (almost pure green #00ff00)
	pub const BLUE: PaletteColor = Self { id: 45 };
	pub const CYAN: PaletteColor = Self { id: 90 }; // #00fcca (almost pure yellow #00ffff)
	pub const MAGENTA: PaletteColor = Self { id: 53 };
	pub const YELLOW: PaletteColor = Self { id: 13 }; // #fdfd00 (almost pure yellow #ffff00)
}

/// The Mk2 can light a button in multiple different ways
pub enum LightMode {
	/// This is the standard mode. A straight consistent light
	Plain,
	/// A flashing motion On->Off->On->Off->...
	Flash,
	/// A smooth pulse
	Pulse,
}

pub struct LaunchpadMk2Output {
	connection: MidiOutputConnection,
}

impl LaunchpadMk2Output {
	const NAME: &'static str = "Launchy Mk2 Output";

	/// Initiate from an existing midir port
	pub fn from_port(midi_output: MidiOutput, port: &MidiOutputPort) -> anyhow::Result<Self> {
		let connection = midi_output.connect(port, Self::NAME)
				.map_err(|_| anyhow!("Failed to connect to port"))?;
		
		return Ok(Self { connection });
	}

	/// Search the midi devices and choose the first midi device belonging to a Launchpad Mk2
	pub fn guess() -> anyhow::Result<Self> {
		let midi_output = MidiOutput::new(crate::APPLICATION_NAME)
				.context("Couldn't create MidiOutput object")?;

		let port = super::guess_port(&midi_output)
				.context(format!("No {} device found", Self::NAME))?;
		let self_ = Self::from_port(midi_output, &port)
				.context("Couldn't make launchpad obj from port")?;
		return Ok(self_);
	}

	fn send(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
		self.connection.send(bytes)?;
		return Ok(());
	}

	/// This is a function testing various parts of this API by executing various commands in order
	/// to find issues either in this library or in your device
	pub fn test_api(&mut self) -> anyhow::Result<()> {
		self.light_all(PaletteColor { id: 1 })?;
		std::thread::sleep(std::time::Duration::from_millis(250));
		self.light_all(PaletteColor { id: 0 })?;

		self.light(Button::ControlButton { number: 0 }, PaletteColor { id: 5 })?;
		self.light(Button::ControlButton { number: 1 }, PaletteColor { id: 13 })?;
		self.light_rgb(Button::ControlButton { number: 2 }, RgbColor { r: 63, g: 0, b: 63 })?;
		self.light_rgb(Button::ControlButton { number: 3 }, RgbColor { r: 63, g: 63, b: 63 })?;
		self.light(Button::GridButton { x: 0, y: 0 }, PaletteColor { id: 5 })?;
		self.light(Button::GridButton { x: 1, y: 0 }, PaletteColor { id: 13 })?;
		self.light_rgb(Button::GridButton { x: 2, y: 0 }, RgbColor { r: 63, g: 0, b: 63 })?;
		self.light_rgb(Button::GridButton { x: 3, y: 0 }, RgbColor { r: 63, g: 63, b: 63 })?;

		self.flash(Button::GridButton { x: 1, y: 1 }, PaletteColor { id: 13 })?;
		self.pulse(Button::GridButton { x: 1, y: 2 }, PaletteColor { id: 13 })?;
		self.light_multiple(&[
			(Button::GridButton { x: 0, y: 1 }, PaletteColor { id: 13 }),
			(Button::GridButton { x: 0, y: 2 }, PaletteColor { id: 14 }),
		])?;
		self.flash_multiple(&[
			(Button::GridButton { x: 1, y: 1 }, PaletteColor { id: 13 }),
			(Button::GridButton { x: 1, y: 2 }, PaletteColor { id: 14 }),
		])?;
		self.pulse_multiple(&[
			(Button::GridButton { x: 2, y: 1 }, PaletteColor { id: 13 }),
			(Button::GridButton { x: 2, y: 2 }, PaletteColor { id: 14 }),
		])?;

		return Ok(());
	}

	pub fn set_button(&mut self, button: Button, color: PaletteColor, light_mode: LightMode)
			-> anyhow::Result<()> {
		
		assert!(color.id <= 127);
		
		let type_byte = match button {
			Button::GridButton { .. } => 0x90,
			Button::ControlButton { .. } => 0xB0,
		} + match light_mode {
			LightMode::Plain => 0,
			LightMode::Flash => 1,
			LightMode::Pulse => 2,
		};

		return self.send(&[type_byte, Self::encode_button(button), color.id]);
	}

	pub fn set_buttons(&mut self, pairs: &[(Button, PaletteColor)], light_mode: LightMode)
			-> anyhow::Result<()> {
		
		assert!(pairs.len() <= 80); // As per Launchpad documentation
		
		let msg_type_byte = match light_mode {
			LightMode::Plain => 10,
			LightMode::Flash => 35,
			LightMode::Pulse => 40,
		};

		return self.send_multiple(msg_type_byte, pairs.iter()
				.map(|(button, color)| (Self::encode_button(*button), *color)));
	}

	pub fn light_multiple_rgb(&mut self, pairs: &[(Button, RgbColor)]) -> anyhow::Result<()> {
		assert!(pairs.len() <= 80);

		let mut bytes = Vec::with_capacity(8 + 12 * pairs.len());

		bytes.extend(&[240, 0, 32, 41, 2, 24, 11]);
		for (button, color) in pairs {
			assert!(color.is_valid());
			bytes.extend(&[Self::encode_button(*button), color.r, color.g, color.b]);
		}
		bytes.push(247);

		return self.send(&bytes);
	}

	pub fn light_columns(&mut self, pairs: &[(u8, PaletteColor)]) -> anyhow::Result<()> {
		assert!(pairs.len() <= 9);

		return self.send_multiple(12, pairs.iter());
	}

	pub fn light_rows<'a>(&mut self, pairs: &'a [(u8, PaletteColor)]) -> anyhow::Result<()> {
		assert!(pairs.len() <= 9);

		return self.send_multiple(13, pairs.iter()
				.map(|(row, color)| (8 - row, *color)));
	}

	pub fn light_all(&mut self, color: PaletteColor) -> anyhow::Result<()> {
		return self.send(&[240, 0, 32, 41, 2, 24, 14, color.id, 247]);
	}

	fn send_multiple<'a, I, T>(&mut self, msg_type_byte: u8, pair_iterator: I) -> anyhow::Result<()>
			where I: Iterator<Item=T>,
			T: std::borrow::Borrow<(u8, PaletteColor)> {
		
		let mut bytes = Vec::with_capacity(8 + 12 * pair_iterator.size_hint().0);

		bytes.extend(&[240, 0, 32, 41, 2, 24, msg_type_byte]);
		for pair in pair_iterator {
			let (button_specifier, color) = pair.borrow();
			bytes.extend(&[*button_specifier, color.id]);
		}
		bytes.push(247);

		return self.send(&bytes);
	}

	pub fn send_clock_tick(&mut self) -> anyhow::Result<()> {
		todo!();
	}

	fn encode_button(button: Button) -> u8 {
		match button {
			Button::GridButton { x, y } => {
				assert!(x <= 8);
				assert!(y <= 7);

				return 10 * (8 - y) + x + 1;
			},
			Button::ControlButton { number } => {
				assert!(number <= 7);

				return number + 104;
			}
		}
	}

	// --------------------------------------------------------------------------------------------
	// Below this point are shorthand function
	// --------------------------------------------------------------------------------------------

	pub fn light(&mut self, button: Button, color: PaletteColor) -> anyhow::Result<()> {
		return self.set_button(button, color, LightMode::Plain);
	}

	/// Starts a flashing motion between the previously shown color on this button and `color`, with
	/// a duty cycle of 50% and a bpm of 120. The bpm can be manually overriden using
	/// `send_clock_tick`. See the documentation there
	pub fn flash(&mut self, button: Button, color: PaletteColor) -> anyhow::Result<()> {
		return self.set_button(button, color, LightMode::Flash);
	}

	pub fn pulse(&mut self, button: Button, color: PaletteColor) -> anyhow::Result<()> {
		return self.set_button(button, color, LightMode::Pulse);
	}

	pub fn light_column(&mut self, column: u8, color: PaletteColor)
			-> anyhow::Result<()> {
		
		return self.light_columns(&[(column, color)]);
	}

	pub fn light_row(&mut self, row: u8, color: PaletteColor)
			-> anyhow::Result<()> {
		
		return self.light_rows(&[(row, color)]);
	}

	pub fn light_rgb(&mut self, button: Button, color: RgbColor) -> anyhow::Result<()> {
		return self.light_multiple_rgb(&[(button, color)]);
	}

	pub fn light_multiple(&mut self, pairs: &[(Button, PaletteColor)]) -> anyhow::Result<()> {
		return self.set_buttons(pairs, LightMode::Plain);
	}

	pub fn flash_multiple(&mut self, pairs: &[(Button, PaletteColor)]) -> anyhow::Result<()> {
		return self.set_buttons(pairs, LightMode::Flash);
	}

	pub fn pulse_multiple(&mut self, pairs: &[(Button, PaletteColor)]) -> anyhow::Result<()> {
		return self.set_buttons(pairs, LightMode::Pulse);
	}
}