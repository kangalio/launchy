use anyhow::{anyhow, Context};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

use crate::Button;


#[derive(Debug, Copy, Clone)]
pub struct Color {
	pub red: u8,
	pub green: u8,
}

impl Color {
	pub const BLACK: Color = Color { red: 0, green: 0 };
	pub const RED: Color = Color { red: 3, green: 0 };
	pub const GREEN: Color = Color { red: 0, green: 3 };
	pub const YELLOW: Color = Color { red: 3, green: 3 };

	pub fn make(red: u8, green: u8) -> Color {
		assert!(red < 4);
		assert!(green < 4);

		return Color { red, green };
	}
}

#[derive(Debug)]
pub enum Brightness {
	Off, Low, Medium, Full
}

pub struct LaunchpadSOutput {
	connection: MidiOutputConnection,
}

impl LaunchpadSOutput {
	const NAME: &'static str = "Launchy S Output";

	pub fn from_port(midi_output: MidiOutput, port: &MidiOutputPort) -> anyhow::Result<Self> {
		let connection = midi_output.connect(port, Self::NAME)
				.map_err(|_| anyhow!("Failed to connect to port"))?;
		
		return Ok(Self { connection });
	}

	pub fn guess() -> anyhow::Result<Self> {
		let midi_output = MidiOutput::new(crate::APPLICATION_NAME)
				.context("Couldn't create MidiOutput object")?;

		let port = super::guess_port(&midi_output)
				.context("No Launchpad S device found")?;
		let self_ = Self::from_port(midi_output, &port)
				.context("Couldn't make launchpad obj from port")?;
		return Ok(self_);
	}

	pub fn set_button(&mut self, button: &Button, color: Color, copy: bool, clear: bool)
			-> anyhow::Result<()> {
		
		let color_code = (color.green << 4)
					| ((clear as u8) << 3)
					| ((copy as u8) << 2)
					| color.red;

		match button {
			Button::GridButton { x, y } => {
				let button_code = y * 16 + x;
				self.connection.send(&[0x90, button_code, color_code])?;
			},
			Button::ControlButton { number } => {
				let button_code = 104 + number;
				self.connection.send(&[0xB0, button_code, color_code])?;
			}
		}

		return Ok(());
	}

	/// All LEDs are turned off, and the mapping mode, buffer settings, and duty cycle are reset to
	/// their default values.
	pub fn reset(&mut self) -> anyhow::Result<()> {
		return self.turn_on_all_leds(Brightness::Off);
	}

	/// Turns on all LEDs to a certain brightness, dictated by the `brightness` parameter.
	/// According to the Launchpad documentation, sending this command resets all other data - see
	/// the `reset()` for more information. However, in my experience, it also sometimes happens.
	/// Weird.
	/// Btw this function is not really intended for regular use. It's more like a test function to
	/// check if the device is working correctly, diagnostic stuff like that.
	pub fn turn_on_all_leds(&mut self, brightness: Brightness) -> anyhow::Result<()> {
		let brightness_code = match brightness {
			Brightness::Off => 0,
			Brightness::Low => 125,
			Brightness::Medium => 126,
			Brightness::Full => 127,
		};

		self.connection.send(&[0xB0, 0x00, brightness_code])?;
		return Ok(());
	}

	/// Launchpad controls the brightness of its LEDs by continually switching them on and off
	/// faster than the eye can see: a technique known as multiplexing. This command provides a way
	/// of altering the proportion of time for which the LEDs are on while they are in low- and
	/// medium-brightness modes. This proportion is known as the duty cycle.
	/// Manipulating this is useful for fade effects, for adjusting contrast, and for creating
	/// custom palettes.
	/// The default duty cycle is 1/5 meaning that low-brightness LEDs are on for only every fifth
	/// multiplex pass, and medium-brightness LEDs are on for two passes in every five.
	/// Generally, lower duty cycles (numbers closer to zero) will increase contrast between
	/// different brightness settings but will also increase flicker; higher ones will eliminate
	/// flicker, but will also reduce contrast. Note that using less simple ratios (such as 3/17 or
	/// 2/11) can also increase	perceived flicker.
	/// If you are particularly sensitive to strobing lights, please use this command with care when
	/// working with large areas of low-brightness LEDs: in particular, avoid duty cycles of 1/8 or
	/// less.
	pub fn set_duty_cycle(&mut self, numerator: u8, denominator: u8) -> anyhow::Result<()> {
		assert!(numerator >= 1);
		assert!(numerator <= 16);
		assert!(denominator >= 3);
		assert!(denominator <= 18);

		if numerator < 9 {
			self.connection.send(&[0xB0, 0x1E, 16 * (numerator - 1) + denominator - 3])?;
		} else {
			self.connection.send(&[0xB0, 0x1F, 16 * (numerator - 9) + denominator - 3])?;
		}
		return Ok(());
	}
}