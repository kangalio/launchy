use anyhow::{anyhow, Context};
use midir::{MidiInput, MidiInputConnection, MidiInputPort};
use std::convert::TryInto;

use crate::Button;


#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Message {
	Press { button: Button },
	Release { button: Button },
	TextEndedOrLooped,
	DeviceInquiry { device_id: u8, firmware_revision: u32 },
	VersionInquiry { bootloader_version: u32, firmware_version: u32 },
	FaderChange { index: u8, value: u8 },
}

pub struct LaunchpadMk2Input;

fn decode_short_message(data: &[u8]) -> Message {
	assert_eq!(data.len(), 3); // if this function was called, it should be

	// first byte of a launchpad midi message is the message type
	match data[0] {
		0x90 => { // Note on
			let button = decode_grid_button(data[1]);
			
			let velocity = data[2];
			match velocity {
				0 => return Message::Release { button },
				127 => return Message::Press { button },
				other => panic!("Unexpected grid note-on velocity {}", other),
			}
		},
		0xB0 => { // Controller change
			match data[1] {
				104..=111 => {
					let button = Button::ControlButton { number: data[1] - 104 };
	
					let velocity = data[2];
					match velocity {
						0 => return Message::Release { button },
						127 => return Message::Press { button },
						other => panic!("Unexpected control note-on velocity {}", other),
					}
				},
				21..=28 => {
					return Message::FaderChange { index: data[1] - 21, value: data[2] };
				},
				_ => panic!("Unexpected data byte 1. {:?}", data),
			}
		},
		// This is the note off code BUT it's not used by the launchpad. It sends zero-velocity
		// note-on messages instead
		0x80 => panic!("Unexpected note-on message: {:?}", data),
		_other => panic!("First byte of midi short messages was unexpected. {:?}", data),
	}
}

fn decode_sysex_message(data: &[u8]) -> Message {
	return match data {
		&[240, 0, 32, 41, 2, 24, 21, 247] => Message::TextEndedOrLooped,
		&[240, 126, device_id, 6, 2, 0, 32, 41, 105, 0, 0, 0, fr1, fr2, fr3, fr4, 247] => {
			let firmware_revision = u32::from_be_bytes([fr1, fr2, fr3, fr4]);
			Message::DeviceInquiry { device_id, firmware_revision }
		}
		&[240, 0, 32, 41, 0, 112, ref data @ .., 247] => {
			let data: [u8; 12] = data.try_into()
					.expect("Invalid version inquiry response length");
			
			let bootloader_version =
					data[0] as u32 * 10000 +
					data[1] as u32 * 1000 +
					data[2] as u32 * 100 +
					data[3] as u32 * 10 +
					data[4] as u32;
			
			let firmware_version =
					data[5] as u32 * 10000 +
					data[6] as u32 * 1000 +
					data[7] as u32 * 100 +
					data[8] as u32 * 10 +
					data[9] as u32;
			
			// Last two bytes are [13, 1] in my case, but the actual meaning of it is unknown.
			// Let's just ignore them here

			Message::VersionInquiry { bootloader_version, firmware_version }
		},
		other => panic!("Unexpected sysex message: {:?}", other),
	}
}

fn decode_grid_button(btn: u8) -> Button {
	let x = (btn % 10) - 1;
	let y = 8 - (btn / 10);
	return Button::GridButton { x, y };
}

impl crate::InputDevice for LaunchpadMk2Input {
	const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad MK2";
	const MIDI_CONNECTION_NAME: &'static str = "Launchy Mk2 Input";
	type Message = Message;

	fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
		if data.len() == 3 {
			return decode_short_message(data);
		} else {
			return decode_sysex_message(data);
		}
	}
}
