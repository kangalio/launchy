use crate::ok_or_continue;
use anyhow::{Result, Context, anyhow};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort,
			MidiInput, MidiInputConnection, MidiInputPort};


fn guess_port<T: midir::MidiIO>(midi_io: &T, keyword: &str) -> Option<T::Port> {
	for port in midi_io.ports() {
		let name = ok_or_continue!(midi_io.port_name(&port));
		
		if name.contains(keyword) {
			return Some(port);
		}
	}

	return None;
}

pub trait OutputDevice where Self: Sized {
	const MIDI_CONNECTION_NAME: &'static str;
	const MIDI_DEVICE_KEYWORD: &'static str;

	/// Initiate from an existing midir connection.
	fn from_connection(connection: MidiOutputConnection) -> anyhow::Result<Self>;

	fn send(&mut self, bytes: &[u8]) -> anyhow::Result<()>;

	fn guess() -> anyhow::Result<Self> {
		let midi_output = MidiOutput::new(crate::APPLICATION_NAME)
				.context("Couldn't create MidiOutput object")?;

		let port = guess_port(&midi_output, Self::MIDI_DEVICE_KEYWORD)
				.context(format!("No '{}' output device found", Self::MIDI_DEVICE_KEYWORD))?;
		
		let connection = midi_output
				.connect(&port, Self::MIDI_CONNECTION_NAME)
				.map_err(|_| anyhow!("Failed to connect to port"))?;
		
		return Self::from_connection(connection);
	}
}

pub trait InputDevice<'a> where Self: Sized {
	const MIDI_CONNECTION_NAME: &'static str;
	const MIDI_DEVICE_KEYWORD: &'static str;
	type Message;

	#[must_use = "If not saved, the connection will be immediately dropped"]
	fn from_port<F>(midi_input: MidiInput, port: &MidiInputPort, user_callback: F)
			-> anyhow::Result<Self>
			where F: FnMut(Self::Message) + Send + 'a;
	
	/// Search the midi devices and choose the first midi device matching the wanted Launchpad type.
	#[must_use = "If not saved, the connection will be immediately dropped"]
	fn guess<F>(user_callback: F) -> anyhow::Result<Self>
			where F: FnMut(Self::Message) + Send + 'a {
		
		let midi_input = MidiInput::new(crate::APPLICATION_NAME)
				.context("Couldn't create MidiInput object")?;

		let port = guess_port(&midi_input, Self::MIDI_DEVICE_KEYWORD)
				.context(format!("No '{}' input device found", Self::MIDI_DEVICE_KEYWORD))?;
		
		return Self::from_port(midi_input, &port, user_callback);
	}
}