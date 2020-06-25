use midir::MidiOutputConnection;

use crate::OutputDevice;
use crate::Button;

/// ## Double buffering
/// To make more economical use of data, the Launchpad has a feature called double buffering.
/// Essentially, Launchpad manages two sets of LED data - buffers - for each pad. By default, these
/// are configured so that the buffer that is updated by incoming MIDI messages is the same as the
/// one that is visible, so that note-on messages immediately change their respective pads. However,
/// the buffers can also be configured so that Launchpad’s LED status is updated invisibly. With a
/// single command, these buffers can then be swapped. The pads will instantly update to show their
/// pre-programmed state, while the pads can again be updated invisibly ready for the next swap. The
/// visible buffer can alternatively be configured to swap automatically at 280ms intervals in order
/// to configure LEDs to flash.

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub struct Color {
	red: u8,
	green: u8,
}

impl Color {
	pub const BLACK: Color = Color { red: 0, green: 0 };
	pub const RED: Color = Color { red: 3, green: 0 };
	pub const GREEN: Color = Color { red: 0, green: 3 };
	pub const YELLOW: Color = Color { red: 3, green: 3 };

	pub fn new(red: u8, green: u8) -> Color {
		assert!(red < 4);
		assert!(green < 4);

		return Color { red, green };
	}

	pub fn red(&self) -> u8 { self.red }
	pub fn green(&self) -> u8 { self.green }
	pub fn set_red(&mut self, red: u8) { assert!(red < 4); self.red = red }
	pub fn set_green(&mut self, green: u8) { assert!(green < 4); self.green = green }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Brightness {
	Off, Low, Medium, Full
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
#[repr(u8)]
pub enum Buffer {
	Buffer0 = 0,
	Buffer1 = 1,
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum DoubleBufferingBehavior {
	/// Only write to the currently edited buffer
	None,
	/// Clear the other buffer's copy of this LED
	Clear,
	/// Write this LED data to both buffers
	Copy,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct DoubleBuffering {
	pub copy: bool,
	pub flash: bool,
	pub edited_buffer: Buffer,
	pub displayed_buffer: Buffer,
}

fn make_color_code(color: Color, dbb: DoubleBufferingBehavior)
		-> u8 {
	
	// Bit 6 - Must be 0
	// Bit 5..4 - Green LED brightness
	// Bit 3 - Clear - If 1: clear the other buffer’s copy of this LED.
	// Bit 2 - Copy - If 1: write this LED data to both buffers.
	// Bit 1..0 - Red LED brightness
	let double_buffering_code = match dbb {
		DoubleBufferingBehavior::None => 0b00,
		DoubleBufferingBehavior::Copy => 0b01,
		DoubleBufferingBehavior::Clear => 0b10,
	};
	return (color.green << 4) | (double_buffering_code << 2) | color.red;
}

pub struct LaunchpadSOutput {
	connection: MidiOutputConnection,
}

impl crate::OutputDevice for LaunchpadSOutput {
	const MIDI_CONNECTION_NAME: &'static str = "Launchy S output";
	const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad S";

	fn from_connection(connection: MidiOutputConnection) -> anyhow::Result<Self> {
		return Ok(Self { connection });
	}

	fn send(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
		self.connection.send(bytes)?;
		return Ok(());
	}
}

impl LaunchpadSOutput {
	/// Updates the state for a single LED, specified by `button`. The color, as well as the double
	/// buffering attributes, are specified in `light_state`.
	pub fn set_button(&mut self, button: Button, color: Color, d: DoubleBufferingBehavior)
			-> anyhow::Result<()> {
		
		let light_code = make_color_code(color, d);

		match button {
			Button::GridButton { x, y } => {
				let button_code = y * 16 + x;
				self.send(&[0x90, button_code, light_code])?;
			},
			Button::ControlButton { index } => {
				let button_code = 104 + index;
				self.send(&[0xB0, button_code, light_code])?;
			}
		}

		return Ok(());
	}

	/// In order to make maximum use of the original Launchpad's slow midi speeds, a rapid LED
	/// lighting mode was invented which allows the lighting of two leds in just a single message.
	/// To use this mode, simply start sending these message and the Launchpad will update the 8x8 
	/// grid in left-to-right, top-to-bottom order, then the eight scene launch buttons in
	/// top-to-bottom order, and finally the eight Automap/Live buttons in left-to-right order
	/// (these are otherwise inaccessible using note-on messages). Overflowing data will be ignored.
	/// 
	/// To leave the mode, simply send any other message. Sending another kind of message and then
	/// re-sending this message will reset the cursor to the top left of the grid.
	pub fn set_button_rapid(&mut self,
		color1: Color, dbb1: DoubleBufferingBehavior,
		color2: Color, dbb2: DoubleBufferingBehavior,
	) -> anyhow::Result<()> {
		
		return self.send(&[0xB2, make_color_code(color1, dbb1), make_color_code(color2, dbb2)]);
	}

	/// Turns on all LEDs to a certain brightness, dictated by the `brightness` parameter.
	/// According to the Launchpad documentation, sending this command resets various configuration
	/// settings - see `reset()` for more information. However, in my experience, that only
	/// sometimes happens. Weird.
	/// 
	/// Btw this function is not really intended for regular use. It's more like a test function to
	/// check if the device is working correctly, diagnostic stuff like that.
	pub fn turn_on_all_leds(&mut self, brightness: Brightness) -> anyhow::Result<()> {
		let brightness_code = match brightness {
			Brightness::Off => 0,
			Brightness::Low => 125,
			Brightness::Medium => 126,
			Brightness::Full => 127,
		};

		return self.send(&[0xB0, 0, brightness_code]);
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
	/// 2/11) can also increase perceived flicker.
	/// If you are particularly sensitive to strobing lights, please use this command with care when
	/// working with large areas of low-brightness LEDs: in particular, avoid duty cycles of 1/8 or
	/// less.
	pub fn set_duty_cycle(&mut self, numerator: u8, denominator: u8) -> anyhow::Result<()> {
		assert!(numerator >= 1);
		assert!(numerator <= 16);
		assert!(denominator >= 3);
		assert!(denominator <= 18);

		if numerator < 9 {
			return self.send(&[0xB0, 30, 16 * (numerator - 1) + (denominator - 3)]);
		} else {
			return self.send(&[0xB0, 31, 16 * (numerator - 9) + (denominator - 3)]);
		}
	}

	/// This method controls the double buffering mode on the Launchpad. See the module
	/// documentation for an explanation on double buffering.
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
	pub fn control_double_buffering(&mut self, d: DoubleBuffering) -> anyhow::Result<()> {
		let last_byte = 0b00100000
				| ((d.copy as u8) << 4)
				| ((d.flash as u8) << 3)
				| ((d.edited_buffer as u8) << 2)
				| d.displayed_buffer as u8;
		
		return self.send(&[0xB0, 0, last_byte]);
	}

	// ------------------------------------------------------
	// Below here are shorthand functions
	// ------------------------------------------------------

	/// All LEDs are turned off, and the mapping mode, buffer settings, and duty cycle are reset to
	/// their default values.
	pub fn reset(&mut self) -> anyhow::Result<()> {
		return self.turn_on_all_leds(Brightness::Off);
	}

	pub fn light(&mut self, button: Button, color: Color) -> anyhow::Result<()> {
		return self.set_button(button, color, DoubleBufferingBehavior::Copy);
	}

	pub fn light_all(&mut self, color: Color) -> anyhow::Result<()> {
		let dbb = DoubleBufferingBehavior::Copy; // this is _probably_ a good default

		for _ in 0..40 {
			self.set_button_rapid(color, dbb, color, dbb)?;
		}
		
		return Ok(());
	}
}

fn make_color_code_loopable(color: Color, should_loop: bool)
		-> u8 {
	
	// Bit 6 - Loop - If 1: loop the text
	// Bit 5..4 - Green LED brightness
	// Bit 3 - uhhhh, I think these should probably be empty?
	// Bit 2 - same as above
	// Bit 1..0 - Red LED brightness
	
	return ((should_loop as u8) << 6) | (color.green() << 4) | color.red();
}

impl LaunchpadSOutput {
	// TODO: fix this
	// Uncommented because I have no idea to parse the return format
	// pub fn request_device_inquiry(&mut self) -> anyhow::Result<()> {
	// 	return self.send(&[240, 126, 127, 6, 1, 247]);
	// }

	pub fn scroll_text(&mut self, text: &[u8], color: Color, should_loop: bool)
			-> anyhow::Result<()> {
		
		let color_code = make_color_code_loopable(color, should_loop);

		let bytes = &[
			&[240, 0, 32, 41, 9, color_code],
			text,
			&[247]
		].concat();

		return self.send(bytes);
	}
}

// TODO: optimize the Launchpad S canvas implementation by utilizing the rapid LED update feature.
// Basically, I need to check what's more efficient: lighting all LEDs individually, or refreshing
// the entire screen using rapid update (even if only some lights changes), or rapidly updating a 
// part of the screen, and individually lighting the rest. To find that out, utilize the code
// snippets below:
// 
// fn x_y_to_rapid_update_index(x: u32, y: u32) -> u32 {
// 	if y >= 1 && x <= 7 {
// 		return (y - 1) * 8 + x;
// 	} else if x == 8 {
// 		return 64 + (y - 1);
// 	} else if y == 0 {
// 		return 72 + x;
// 	} else {
// 		panic!("We didn't even do bounds checking but ({}|{}) still managed to fail", x, y);
// 	}
// }

// let mut changes: Vec<_> = changes.iter()
// 				.map(|&(x, y, color)| (x_y_to_rapid_update_index(x, y), x, y, color))
// 				.collect();
// 		changes.sort_unstable_by_key(|&(rapid_update_index, ..)| rapid_update_index);

impl crate::Flushable for LaunchpadSOutput {
	const BOUNDING_BOX_WIDTH: u32 = 9;
	const BOUNDING_BOX_HEIGHT: u32 = 9;

	fn is_valid(x: u32, y: u32) -> bool {
		if x > 8 || y > 8 { return false }
		if x == 8 && y == 0 { return false }
		return true;
	}

	fn flush(&mut self, changes: &[(u32, u32, crate::Color)]) -> anyhow::Result<()> {
		for &(x, y, color) in changes {
			let (r, g, _b) = color.quantize(4);
			self.light(crate::Button::from_abs(x as u8, y as u8), Color::new(r, g))?;
		}

		return Ok(());
	}
}

pub type Canvas = crate::GenericCanvas<LaunchpadSOutput>;