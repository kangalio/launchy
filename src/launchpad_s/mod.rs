use anyhow::{anyhow, Context};
use crate::ok_or_continue;

mod input;
pub use input::*;

mod output;
pub use output::*;


fn guess_port<T: midir::MidiIO>(midi_io: &T) -> Option<T::Port> {
	for port in midi_io.ports() {
		let name = ok_or_continue!(midi_io.port_name(&port));
		
		if name.contains("Launchpad S") {
			return Some(port);
		}
	}

	return None;
}