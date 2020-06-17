use anyhow::{anyhow, Context};
use midir::{MidiInput, MidiInputConnection, MidiInputPort};

use crate::Button;


#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Message {
	Press { button: Button },
	Release { button: Button },
}

pub struct LaunchpadMk2Input<'a> {
	_connection: MidiInputConnection<'a, ()>,
}

impl<'a> LaunchpadMk2Input<'a> {
	const NAME: &'static str = "Launchy Mk2 Input";

	pub fn from_port<F>(midi_input: MidiInput, port: &MidiInputPort, mut user_callback: F)
			-> anyhow::Result<Self>
			where F: FnMut(Message) + Send + 'a {
		
		let midir_callback = move |timestamp: u64, data: &[u8], _: &mut _| {
			let msg = Self::decode_message(timestamp, data);
			(user_callback)(msg);
		};
		
		let connection = midi_input.connect(port, Self::NAME, midir_callback, ())
				.map_err(|_| anyhow!("Failed to connect to port"))?;
		
		return Ok(Self { _connection: connection });
	}

	pub fn guess<F>(callback: F) -> anyhow::Result<Self>
			where F: FnMut(Message) + Send + 'a {
		
		let midi_in = MidiInput::new(crate::APPLICATION_NAME)
				.context("Couldn't create MidiInput object")?;

		let port = super::guess_port(&midi_in)
				.context(format!("No {} device found", Self::NAME))?;
		let self_ = Self::from_port(midi_in, &port, callback)
				.context("Couldn't make launchpad input obj from port")?;
		return Ok(self_);
	}

	fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
		assert_eq!(data.len(), 3);

		// first byte of a launchpad midi message is the message type
		match data[0] {
			0x90 => { // Note on
				let button = Self::decode_grid_button(data[1]);
				
				let velocity = data[2];
				match velocity {
					0 => return Message::Release { button },
					127 => return Message::Press { button },
					other => panic!("Unexpected grid note-on velocity {}", other),
				}
			},
			0xB0 => { // Controller change
				let button = Button::ControlButton { number: data[1] - 104 };

				let velocity = data[2];
				match velocity {
					0 => return Message::Release { button },
					127 => return Message::Press { button },
					other => panic!("Unexpected control note-on velocity {}", other),
				}
			},
			// This is the note off code BUT it's not used by the launchpad. It sends zero-velocity
			// note-on messages instead
			0x80 => panic!("Unexpected note-on message: {:?}", data),
			_other => panic!("First byte of midi short messages was unexpected. {:?}", data),
		}
	}

	fn decode_grid_button(btn: u8) -> Button {
		let x = (btn % 10) - 1;
		let y = 8 - (btn / 10);
		return Button::GridButton { x, y };
	}
} 
