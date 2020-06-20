use anyhow::{anyhow, Context};
use midir::{MidiInput, MidiInputConnection, MidiInputPort};

use crate::Button;


#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Message {
	Press { button: Button },
	Release { button: Button },
	TextEndedOrLooped,
	DeviceInquiry { device_id: u8, firmware_revision: u32 },
	FaderChange { index: u8, value: u8 },
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum WaitFor {
	TextEnded, DeviceInquiry
}

pub struct LaunchpadMk2Input<'a> {
	_connection: MidiInputConnection<'a, ()>,
	wait_release_receiver: std::sync::mpsc::Receiver<WaitFor>,
}

impl<'a> crate::InputDevice<'a> for LaunchpadMk2Input<'a> {
	const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad MK2";
	const MIDI_CONNECTION_NAME: &'static str = "Launchy Mk2 Input";
	type Message = Message;

	fn from_port<F>(midi_input: MidiInput, port: &MidiInputPort, mut user_callback: F)
			-> anyhow::Result<Self>
			where F: FnMut(Message) + Send + 'a {
		
		let (wait_release_sender, wait_release_receiver) = std::sync::mpsc::channel();
		let midir_callback = move |timestamp: u64, data: &[u8], _: &mut _| {
			let msg = match Self::decode_message(timestamp, data) {
				Some(msg) => msg,
				None => return, // guess the msg was invalid or unimplemnted or smth. Let's skip it
			};

			// notify object that a message arrived that might be of interest if someone is waiting
			// for a specific message to arrive
			let wait_release_token = match msg {
				Message::TextEndedOrLooped => Some(WaitFor::TextEnded),
				Message::DeviceInquiry { .. } => Some(WaitFor::DeviceInquiry),
				_ => None,
			};
			if let Some(wait_release_token) = wait_release_token {
				wait_release_sender.send(wait_release_token)
						.expect("Other end of the wait release channel has hung up...? 
								(shouldn't happen)");
			}

			(user_callback)(msg);
		};
		
		let connection = midi_input.connect(port, Self::MIDI_CONNECTION_NAME, midir_callback, ())
				.map_err(|_| anyhow!("Failed to connect to port"))?; // can't use context()
		
		return Ok(Self { _connection: connection, wait_release_receiver });
	}
}

impl LaunchpadMk2Input<'_> {
	// ...jank?
	pub fn wait_for(&self, anticipated_token: WaitFor) {
		loop {
			// wait until a new message arrived
			let received_token = self.wait_release_receiver.recv()
					.expect("Opposite has hung up :/ shouldn't happen");
			
			if received_token == anticipated_token {
				return;
			}
		}
	}

	fn decode_short_message(data: &[u8]) -> Message {
		assert_eq!(data.len(), 3); // if this function was called, it should be

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

	fn decode_sysex_message(data: &[u8]) -> Option<Message> {
		match data {
			&[240, 0, 32, 41, 2, 24, 21, 247] => return Some(Message::TextEndedOrLooped),
			&[240, 126, device_id, 6, 2, 0, 32, 41, 105, 0, 0, 0, fr1, fr2, fr3, fr4, 247] => {
				let firmware_revision = u32::from_be_bytes([fr1, fr2, fr3, fr4]);
				return Some(Message::DeviceInquiry { device_id, firmware_revision });
			}
			&[240, 0, 32, 41, 0, 112, ref _data @ .., 247] => {
				// let data: [u8; 12] = data.try_into()
				// 		.expect("Invalid version inquiry response length");
				// TODO: Figure out how to parse the data (it's not in Novation's docs)
				// println!("can't figure out how to parse {:?}", data);

				return None;
			},
			other => panic!("Unexpected sysex message: {:?}", other),
		}
	}

	fn decode_message(_timestamp: u64, data: &[u8]) -> Option<Message> {
		if data.len() == 3 {
			return Some(Self::decode_short_message(data));
		} else {
			return Self::decode_sysex_message(data);
		}
	}

	fn decode_grid_button(btn: u8) -> Button {
		let x = (btn % 10) - 1;
		let y = 8 - (btn / 10);
		return Button::GridButton { x, y };
	}
}
