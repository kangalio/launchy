use anyhow::{anyhow, Context};
use midir::{MidiInput, MidiInputConnection, MidiInputPort};

use crate::Button;


#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Message {
	Press { button: Button },
	Release { button: Button },
	TextEndedOrLooped,
}

fn decode_grid_button(btn: u8) -> Button {
	return Button::GridButton { x: btn % 16, y: btn / 16 };
}

pub struct LaunchpadSInput<'a> {
	// yes the connection is never explictly used BUT we need it in here to uphold the connection
	#[allow(dead_code)]
	connection: MidiInputConnection<'a, ()>,
}

impl<'a> crate::InputDevice<'a> for LaunchpadSInput<'a> {
	const MIDI_CONNECTION_NAME: &'static str = "Launchy S input";
	const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad S";
	type Message = Message;

	fn from_port<F>(midi_input: MidiInput, port: &MidiInputPort, mut user_callback: F)
			-> anyhow::Result<Self>
			where F: FnMut(Self::Message) + Send + 'a {
		
		let midir_callback = move |timestamp: u64, data: &[u8], _: &mut _| {
			let msg = Self::decode_message(timestamp, data);
			(user_callback)(msg);
		};
		
		let connection = midi_input.connect(port, Self::MIDI_CONNECTION_NAME, midir_callback, ())
				.map_err(|_| anyhow!("Failed to connect to port"))?;
		
		return Ok(Self { connection });
	}
}

impl LaunchpadSInput<'_> {
	fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
		// first byte of a launchpad midi message is the message type
		match data {
			[0x90, button, velocity] => { // Note on
				let button = decode_grid_button(data[1]);
				
				let velocity = data[2];
				match velocity {
					0 => return Message::Release { button },
					127 => return Message::Press { button },
					other => panic!("Unexpected grid note-on velocity {}", other),
				}
			},
			[0xB0, number @ 104..=111, velocity] => { // Controller change
				let button = Button::ControlButton { number: number - 104 };

				match velocity {
					0 => return Message::Release { button },
					127 => return Message::Press { button },
					other => panic!("Unexpected control note-on velocity {}", other),
				}
			},
			[0xB0, 0, 3] => return Message::TextEndedOrLooped,
			// YES we have no note off message handler here because it's not used by the launchpad.
			// It sends zero-velocity note-on messages instead.
			_ => panic!("Unexpected midi message: {:?}", data),
		}
	}
}
