use crate::ok_or_continue;

mod input;
pub use input::*;

// mod output;
// pub use output::*;


// pub enum Color {
// 	/// A color from the Mk2 color palette. See the "Launchpad MK2 Programmers Reference Manual"
// 	/// to see the palette.
// 	/// Must be within `0..=127`
// 	Palette(u8),
// 	/// An RGB color. Each component may only go up to 63
// 	Rgb { r: u8, g: u8, b: u8 },
// }

fn guess_port<T: midir::MidiIO>(midi_io: &T) -> Option<T::Port> {
	for port in midi_io.ports() {
		let name = ok_or_continue!(midi_io.port_name(&port));
		
		if name.contains("Launchpad MK2") {
			return Some(port);
		}
	}

	return None;
}