use anyhow::{anyhow, Context};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

use crate::Button;
use crate::OutputDevice;

pub use crate::capabilities::original::*;


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

impl OriginalLaunchpad for LaunchpadSOutput {}

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