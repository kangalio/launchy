use anyhow::{anyhow, Context};
use midir::{MidiInput, MidiInputConnection, MidiInputPort};
use crate::{ok_or_continue};


#[derive(Debug)]
pub enum Button {
	ControlButton { number: u8 },
	GridButton { x: u8, y: u8 },
}

#[derive(Debug)]
pub enum Message {
	NoteOff { button: Button },
	NoteOn { button: Button },
}

pub struct LaunchpadSInput<'a> {
	_connection: MidiInputConnection<'a, ()>,
}

impl<'a> LaunchpadSInput<'a> {
	const NAME: &'static str = "LaunchpadRs Launchpad S"; // dunno what this should be lol

	pub fn from_port<F>(midi_input: MidiInput, port: &MidiInputPort, user_callback: F)
			-> anyhow::Result<Self>
			where F: Fn(Message) + Send + 'a {
		
		let midir_callback = move |timestamp: u64, data: &[u8], _: &mut _| {
			let msg = Self::decode_message(timestamp, data);
			(user_callback)(msg);
		};
		
		let connection = midi_input.connect(port, Self::NAME, midir_callback, ())
				.map_err(|_| anyhow!("Failed to connect to port"))?;
		
		return Ok(Self { _connection: connection });
	}

	pub fn guess<F>(callback: F) -> anyhow::Result<Self>
			where F: Fn(Message) + Send + 'a {
		
		let midi_in = MidiInput::new(crate::APPLICATION_NAME)
				.context("Couldn't create MidiInput object")?;

		for port in midi_in.ports() {
			let name = ok_or_continue!(midi_in.port_name(&port));
			
			if name.contains("Launchpad S") {
				let object = Self::from_port(midi_in, &port, callback)
						.context("Couldn't make launchpad input obj from port")?;
				return Ok(object);
			}
		}
	
		return Err(anyhow!("No Launchpad S input found"));
	}

	fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
		assert_eq!(data.len(), 3);

		// first byte of a launchpad midi message is the message type
		match data[0] {
			0x90 => { // Note on
				let button = Self::decode_grid_button(data[1]);
				
				let velocity = data[2];
				match velocity {
					0 => return Message::NoteOff { button },
					127 => return Message::NoteOn { button },
					other => panic!("Unexpected grid note-on velocity {}", other),
				}
			},
			0xB0 => { // Controller change
				let button = Button::ControlButton { number: data[1] - 104 };

				let velocity = data[2];
				match velocity {
					0 => return Message::NoteOff { button },
					127 => return Message::NoteOn { button },
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
		return Button::GridButton { x: btn % 16, y: btn / 16 };
	}
} 
