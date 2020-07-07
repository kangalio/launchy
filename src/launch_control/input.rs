use super::{Button, Template};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Knob(u8);

impl Knob {
	pub fn new(index: u8) -> Self {
		assert!(index < 16);
		Self(index)
	}

	pub fn upper(index: u8) -> Self { Self::new(index) }
	pub fn lower(index: u8) -> Self { Self::new(index + 8) }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Message {
	/// When a button is pressed
	Press { template: Template, button: Button },

	/// When a button is released
	Release { template: Template, button: Button },
	
	/// When the user presses a pad button, then changes the template, and then releases the button,
	/// this message will be fired on release. The Launch Control provides no information which
	/// button has been released, nor the template it was pressed or released in
	StalePadRelease,

	/// Same meaning as StalePadRelease, but for control buttons instead of pads
	StaleControlButtonRelease,

	/// When a template is changed using the non-programmatically-accessible template buttons on
	/// the Launch Control
	TemplateChanged { template: Template },

	/// When a knob has been moved
	KnobChanged { template: Template, knob: Knob, value: u8 },
}

pub struct Input;

impl Input {
	fn decode_short_message(data: &[u8]) -> Message {
		let status = data[0] & 0xF0;
		let template = Template(data[0] & 0x0F);
		let note = data[1];
		let velocity = data[2];

		// why am I not sending the template on Stale*Release messages? That's because the device
		// doesn't provide it. the lower 4 bits are always zero on those Stale message, so I can't
		// put it into the Message

		match [status, note, velocity] {
			// Pad buttons press + release
			[0x90, button @ 9..=12, 127] => Message::Press { template, button: Button::pad(button - 9) },
			[0x80, button @ 9..=12, 0] => Message::Release { template, button: Button::pad(button - 9) },
			[0x90, button @ 25..=28, 127] => Message::Press { template, button: Button::pad(button - 25 + 4) },
			[0x80, button @ 25..=28, 0] => Message::Release { template, button: Button::pad(button - 25 + 4) },
			[0x80, 0, 0] => Message::StalePadRelease,

			// Control buttons press + release
			[0xB0, button @ 114..=117, 127] => Message::Press { template, button: Button::control(button - 114) },
			[0xB0, button @ 114..=117, 0] => Message::Release { template, button: Button::control(button - 114) },
			[0xB0, 0, 0] => Message::StaleControlButtonRelease,

			// Knob changes
			[0xB0, knob @ 21..=28, value] => Message::KnobChanged { template, knob: Knob::upper(knob - 21), value },
			[0xB0, knob @ 41..=48, value] => Message::KnobChanged { template, knob: Knob::lower(knob - 41), value },

			_ => panic!("Unexpected short message {:?}", data),
		}
	}

	fn decode_sysex_message(data: &[u8]) -> Message {
		match *data {
			[240, 0, 32, 41, 2, 10, 119, template, 247] => Message::TemplateChanged {
				template: Template(template)
			},
			_ => panic!("Unexpected sysex message {:?}", data),
		}
	}
}

impl crate::InputDevice for Input {
	const MIDI_CONNECTION_NAME: &'static str = "Launchy Launch Control input";
	const MIDI_DEVICE_KEYWORD: &'static str = "Launch Control";
	type Message = Message;

	fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
		match data.len() {
			3 => Self::decode_short_message(data),
			_ => Self::decode_sysex_message(data),
		}
	}
}
