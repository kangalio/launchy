use midir::MidiOutputConnection;

use crate::OutputDevice;
use super::{Button, Template};

#[doc(inline)]
pub use crate::protocols::double_buffering::*;

/// The Launch Control output connection handler.
pub struct Output {
	connection: MidiOutputConnection,
}

impl crate::OutputDevice for Output {
	const MIDI_CONNECTION_NAME: &'static str = "Launchy Launch Control output";
	const MIDI_DEVICE_KEYWORD: &'static str = "Launch Control";

	fn from_connection(connection: MidiOutputConnection) -> Result<Self, crate::MidiError> {
		return Ok(Self { connection });
	}

	fn send(&mut self, bytes: &[u8]) -> Result<(), crate::MidiError> {
		self.connection.send(bytes)?;
		return Ok(());
	}
}

impl Output {
	/// Updates the state for a single LED, specified by `button`. The color, as well as the double
	/// buffering attributes, are specified in `light_state`.
	/// 
	/// The given `template` must match the currently selected template on the Launch Control, or
	/// nothing will happen.
	pub fn set_button(&mut self,
		template: impl Into<Template>,
		button: Button,
		color: Color,
		d: DoubleBufferingBehavior
	) -> Result<(), crate::MidiError> {
		
		let light_code = make_color_code(color, d);
		let status = match button { Button::Pad(_) => 0x90, _ => 0xB0 } + template.into().0;
		self.send(&[status, button.code(), light_code])
	}

	/// The Launch Control interprets this message exactly the same as
	/// `self.set_button(template, button, Color::Black, DoubleBufferingBehavior::None)`
	/// 
	/// The given `template` must match the currently selected template on the Launch Control, or
	/// nothing will happen.
	pub fn turn_off_button(&mut self,
		template: impl Into<Template>,
		button: Button
	) -> Result<(), crate::MidiError> {
		// velocity byte is ignored, so I'm just setting it to zero
		match button {
			Button::Pad(_) => self.send(&[0x80, button.code(), 0]),
			_ => self.set_button(template, button, Color::BLACK, DoubleBufferingBehavior::None),
		}
	}

	/// Light multiple buttons with varying colors and double buffering behavior.
	/// 
	/// The given `template` must match the currently selected template on the Launch Control, or
	/// nothing will happen.
	pub fn light_multiple(&mut self,
		template: impl Into<Template>,
		pads: impl IntoIterator<Item = impl std::borrow::Borrow<(
			Button,
			Color,
			DoubleBufferingBehavior
		)>>,
	) -> Result<(), crate::MidiError> {
		let mut bytes = Vec::new();
		bytes.extend(&[240, 0, 32, 41, 2, 10, 120, template.into().0]);
		for entry in pads.into_iter() {
			let &(btn, color, dbb) = entry.borrow();

			bytes.extend(&[btn.as_index(), make_color_code(color, dbb)]);
		}
		bytes.push(247);

		self.send(&bytes)
	}

	// this doesn't seem to do ANYTHING at all /shrug
	// pub fn toggle(&mut self, template: impl Into<Template>, button: Button, on: bool) -> anyhow::Result<()> {
	// 	let value = if on { 127 } else { 0 };
	// 	self.send(&[240, 0, 32, 41, 2, 10, 123, template.into().0, button.as_index(), value, 247])
	// }

	/// Change the template that will be displayed and edited on
	pub fn change_template(&mut self,
		template: impl Into<Template>
	) -> Result<(), crate::MidiError> {
		self.send(&[240, 0, 32, 41, 2, 10, 119, template.into().0, 247])
	}

	/// Turns on all LEDs to a certain brightness, dictated by the `brightness` parameter.
	/// According to the Launchpad documentation, sending this command resets various configuration
	/// settings - see `reset()` for more information. However, in my experience, that only
	/// sometimes happens. Weird.
	/// 
	/// The `template` parameter specifies for which template this message is intended
	/// 
	/// Btw this function is not really intended for regular use. It's more like a test function to
	/// check if the device is working correctly, diagnostic stuff like that.
	pub fn turn_on_all_leds(&mut self,
		template: impl Into<Template>,
		brightness: Brightness
	) -> Result<(), crate::MidiError> {
		let brightness_code = match brightness {
			Brightness::Off => 0,
			Brightness::Low => 125,
			Brightness::Medium => 126,
			Brightness::Full => 127,
		};

		return self.send(&[0xB0 + template.into().0, 0, brightness_code]);
	}

	/// This method controls the double buffering mode on the Launchpad. ~~See the module
	/// documentation for an explanation on double buffering.~~ // TODO
	/// 
	/// The default state is no flashing; the first buffer is both the update and the displayed
	/// buffer: In this mode, any LED data written to Launchpad is displayed instantly. Sending this
	/// message also resets the flash timer, so it can be used to resynchronise the flash rates of
	/// all the Launchpads connected to a system. 
	/// 
	/// - If `copy` is set, copy the LED states from the new displayed buffer to the new updating
	/// buffer.
	/// - If `flash` is set, continually flip displayed buffers to make selected LEDs flash.
	/// - `updated`: the new updated buffer
	/// - `displayed`: the new displayed buffer
	/// 
	/// The `template` parameter specifies for which template this message is intended
	pub fn control_double_buffering(&mut self,
		template: impl Into<Template>,
		d: DoubleBuffering
	) -> Result<(), crate::MidiError> {

		let last_byte = 0b00100000
				| ((d.copy as u8) << 4)
				| ((d.flash as u8) << 3)
				| ((d.edited_buffer as u8) << 2)
				| d.displayed_buffer as u8;
		
		return self.send(&[0xB0 + template.into().0, 0, last_byte]);
	}

	// -----------------------------
	// Shorthand functions:
	// -----------------------------

	/// All LEDs are turned off, and the mapping mode, buffer settings, and duty cycle are reset to
	/// their default values.
	pub fn reset(&mut self, template: impl Into<Template>) -> Result<(), crate::MidiError> {
		return self.turn_on_all_leds(template, Brightness::Off);
	}

	/// Light a button with a certain color.
	/// 
	/// The given `template` must match the currently selected template on the Launch Control, or
	/// nothing will happen.
	pub fn light(&mut self,
		template: impl Into<Template>,
		button: Button,
		color: Color,
		dbb: DoubleBufferingBehavior
	) -> Result<(), crate::MidiError> {
		self.light_multiple(template, &[(button, color, dbb)])
	}
}