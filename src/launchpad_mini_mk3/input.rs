use core::panic;

pub use crate::protocols::query::*;

use super::Button;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
/// A Launchpad Mini MK3 input message
pub enum Message {
    /// A button was pressed
    Press {
        button: Button,
    },
    /// A button was released
    Release {
        button: Button,
    },
    /// Emitted after a text scroll was initiated
    TextEndedOrLooped,
    /// The response to a [device inquiry request](super::Output::request_device_inquiry)
    DeviceInquiry(DeviceInquiry),
    /// The response to a [version inquiry request](super::Output::request_version_inquiry)
    VersionInquiry(VersionInquiry),
    /// Emitted when a fader was changed by the user, in [fader
    /// mode](super::Output::enter_fader_mode)
    FaderChange {
        index: u8,
        value: u8,
    },

    Unknown,
}

/// The Launchpad Mini MK3 input connection creator.
pub struct Input;

fn decode_grid_button(btn: u8) -> Button {
    let x = (btn % 10) - 1;
    let y = 8 - (btn / 10);
    Button::GridButton { x, y }
}

fn decode_control_button(btn: u8) -> Button {
    // Because of how the buttons are communicated, the right-side control buttons are encoded as
    // 89, 79, 69, 59, 49, 39, 29, 19 and the top control buttons are encoded as 91, 92, 95, 96,
    // 97, 98 This function converts those values to the correct button

    if btn >= 91 && btn <= 98 {
        return Button::ControlButton { index: btn - 91 };
    } else if btn >= 19 && btn <= 89 && btn % 10 == 9 {
        return Button::ControlButton {
            index: 8 + (8 - (btn - 9) / 10),
        };
    }

    panic!("Unexpected control button value {}", btn);
}

impl crate::InputDevice for Input {
    const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad Mini MK3 LPMiniMK3 MI";
    const MIDI_CONNECTION_NAME: &'static str = "Launchy Mini Mk3 Input";
    type Message = Message;

    fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
        if let Some(device_inquiry) = parse_device_query(data) {
            return Message::DeviceInquiry(device_inquiry);
        }

        if let Some(version_inquiry) = parse_version_query(data) {
            return Message::VersionInquiry(version_inquiry);
        }

        match data {
            // Press
            &[0x90, button, velocity] => {
                let button = decode_grid_button(button);

                match velocity {
                    // TODO: Is this release deprecated?
                    0 => Message::Release { button },
                    127 => Message::Press { button },
                    other => panic!("Unexpected grid note-on velocity {}", other),
                }
            }
            // Implement release (actively used)
            &[0x80, button, extra] => {
                // TODO: figure out what extra is, appears to be 0x40 for all buttons
                if extra != 0x40 {
                    panic!("Unexpected grid note-off extra byte {}", extra);
                }

                let button = decode_grid_button(button);

                println!("Extra release");

                Message::Release { button }
            }
            // Control button press & release
            &[0xB0, number, velocity] => {
                let button = decode_control_button(number);

                match velocity {
                    0 => Message::Release { button },
                    127 => Message::Press { button },
                    other => panic!("Unexpected grid note-on velocity {}", other),
                }
            }
            // // Controller change
            // &[0xB0, number @ 104..=111, velocity] => {
            //     let button = Button::ControlButton {
            //         index: number - 104,
            //     };

            //     match velocity {
            //         0 => Message::Release { button },
            //         127 => Message::Press { button },
            //         other => panic!("Unexpected control note-on velocity {}", other),
            //     }
            // }
            // // Fader change
            // &[0xB0, number @ 21..=28, value] => Message::FaderChange {
            //     index: number - 21,
            //     value,
            // },
            // &[240, 0, 32, 41, 2, 24, 21, 247] => Message::TextEndedOrLooped,
            &[240, 0, 32, 41, 2, 13, 14, 1, 247] => {
                println!("Programmer Mode Successfully enabled");
                Message::Unknown
            }
            other => panic!("Unexpected midi message: {:?}", other),
        }
    }
}
