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

pub struct LaunchpadSInput;

impl crate::InputDevice for LaunchpadSInput {
	const MIDI_CONNECTION_NAME: &'static str = "Launchy S input";
	const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad S";
	type Message = Message;

	fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
		// first byte of a launchpad midi message is the message type
		return match data {
			&[0x90, button, velocity] => { // Note on
				let button = decode_grid_button(button);
				
				match velocity {
					0 => Message::Release { button },
					127 => Message::Press { button },
					other => panic!("Unexpected grid note-on velocity {}", other),
				}
			},
			&[0xB0, number @ 104..=111, velocity] => { // Controller change
				let button = Button::ControlButton { index: number - 104 };

				match velocity {
					0 => Message::Release { button },
					127 => Message::Press { button },
					other => panic!("Unexpected control note-on velocity {}", other),
				}
			},
			&[0xB0, 0, 3] => Message::TextEndedOrLooped,
			// YES we have no note off message handler here because it's not used by the launchpad.
			// It sends zero-velocity note-on messages instead.
			other => panic!("Unexpected midi message: {:?}", other),
		};
	}
}
