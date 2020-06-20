#![allow(unused_imports)]

use anyhow::{Result, Context, anyhow};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort,
			MidiInput, MidiInputConnection, MidiInputPort};

mod util;

mod color;
pub use color::*;

mod canvas;
pub use canvas::*;

pub mod launchpad_s;
pub use launchpad_s as s;

pub mod launchpad_mk2;
pub use launchpad_mk2 as mk2;


/// Identifier used for e.g. the midi port names etc.
const APPLICATION_NAME: &'static str = "LaunchpadRs";


#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Button {
	ControlButton { number: u8 }, // TODO: Rename "number" -> "index"
	GridButton { x: u8, y: u8 },
}

impl Button {
	pub const UP: Self = Button::ControlButton { number: 0 };
	pub const DOWN: Self = Button::ControlButton { number: 1 };
	pub const LEFT: Self = Button::ControlButton { number: 2 };
	pub const RIGHT: Self = Button::ControlButton { number: 3 };
	pub const SESSION: Self = Button::ControlButton { number: 4 };
	pub const USER_1: Self = Button::ControlButton { number: 5 };
	pub const USER_2: Self = Button::ControlButton { number: 6 };
	pub const MIXER: Self = Button::ControlButton { number: 7 };
	pub const VOLUME: Self = Button::GridButton { x: 8, y: 0 };
	pub const PAN: Self = Button::GridButton { x: 8, y: 1 };
	pub const SEND_A: Self = Button::GridButton { x: 8, y: 2 };
	pub const SEND_B: Self = Button::GridButton { x: 8, y: 3 };
	pub const STOP: Self = Button::GridButton { x: 8, y: 4 };
	pub const MUTE: Self = Button::GridButton { x: 8, y: 5 };
	pub const SOLO: Self = Button::GridButton { x: 8, y: 6 };
	pub const RECORD_ARM: Self = Button::GridButton { x: 8, y: 7 };

	/// Returns x coordinate assuming coordinate origin in the leftmost control button
	pub fn abs_x(&self) -> u8 {
		match self {
			Self::ControlButton { number } => return *number,
			Self::GridButton { x, .. } => return *x,
		}
	}

	/// Returns y coordinate assuming coordinate origin in the leftmost control button
	pub fn abs_y(&self) -> u8 {
		match self {
			Self::ControlButton { .. } => return 0,
			Self::GridButton { y, .. } => y + 1,
		}
	}
}

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

	fn from_connection(connection: MidiOutputConnection) -> Self;

	fn guess() -> anyhow::Result<Self> {
		let midi_output = MidiOutput::new(crate::APPLICATION_NAME)
				.context("Couldn't create MidiOutput object")?;

		let port = guess_port(&midi_output, Self::MIDI_DEVICE_KEYWORD)
				.context(format!("No '{}' output device found", Self::MIDI_DEVICE_KEYWORD))?;
		
		let connection = midi_output
				.connect(&port, Self::MIDI_CONNECTION_NAME)
				.map_err(|_| anyhow!("Failed to connect to port"))?;
		
		return Ok(Self::from_connection(connection));
	}
}

pub trait InputDevice<'a> where Self: Sized {
	const MIDI_CONNECTION_NAME: &'static str;
	const MIDI_DEVICE_KEYWORD: &'static str;
	type Message;

	fn from_port<F>(midi_input: MidiInput, port: &MidiInputPort, user_callback: F)
			-> anyhow::Result<Self>
			where F: FnMut(Self::Message) + Send + 'a;
	
	fn guess<F>(user_callback: F) -> anyhow::Result<Self>
			where F: FnMut(Self::Message) + Send + 'a {
		
		let midi_input = MidiInput::new(crate::APPLICATION_NAME)
				.context("Couldn't create MidiInput object")?;

		let port = guess_port(&midi_input, Self::MIDI_DEVICE_KEYWORD)
				.context(format!("No '{}' input device found", Self::MIDI_DEVICE_KEYWORD))?;
		
		return Self::from_port(midi_input, &port, user_callback);
	}
}