pub use crate::protocols::query::*;

use super::Button;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
/// A Launchpad MK2 input message
pub enum Message {
    /// A button was pressed
    Press { button: Button },
    /// A button was released
    Release { button: Button },
    /// Emitted after a text scroll was initiated
    TextEndedOrLooped,
    /// The response to a [device inquiry request](super::Output::request_device_inquiry)
    DeviceInquiry(DeviceInquiry),
    /// The response to a [version inquiry request](super::Output::request_version_inquiry)
    VersionInquiry(VersionInquiry),
    /// Emitted when a fader was changed by the user, in [fader
    /// mode](super::Output::enter_fader_mode)
    FaderChange { index: u8, value: u8 },
}

/// The Launchpad MK2 input connection creator.
pub struct Input;

fn decode_grid_button(btn: u8) -> Button {
    let x = (btn % 10) - 1;
    let y = 8 - (btn / 10);
    Button::GridButton { x, y }
}

impl crate::InputDevice for Input {
    const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad MK2";
    const MIDI_CONNECTION_NAME: &'static str = "Launchy Mk2 Input";
    type Message = Message;

    fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
        if let Some(device_inquiry) = parse_device_query(data) {
            return Message::DeviceInquiry(device_inquiry);
        }

        if let Some(version_inquiry) = parse_version_query(data) {
            return Message::VersionInquiry(version_inquiry);
        }

        match data {
            &[0x90, button, velocity] => {
                let button = decode_grid_button(button);

                match velocity {
                    0 => Message::Release { button },
                    127 => Message::Press { button },
                    other => panic!("Unexpected grid note-on velocity {}", other),
                }
            }
            // Controller change
            &[0xB0, number @ 104..=111, velocity] => {
                let button = Button::ControlButton {
                    index: number - 104,
                };

                match velocity {
                    0 => Message::Release { button },
                    127 => Message::Press { button },
                    other => panic!("Unexpected control note-on velocity {}", other),
                }
            }
            // Fader change
            &[0xB0, number @ 21..=28, value] => Message::FaderChange {
                index: number - 21,
                value,
            },
            &[240, 0, 32, 41, 2, 24, 21, 247] => Message::TextEndedOrLooped,
            other => panic!("Unexpected midi message: {:?}", other),
        }
    }
}
