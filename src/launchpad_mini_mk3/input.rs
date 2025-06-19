use core::panic;

pub use crate::protocols::query::*;

use super::{Button, Layout, SleepMode};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
/// A Launchpad Mini MK3 input message
pub enum Message {
    /// A button was pressed
    Press { button: Button },
    /// A button was released
    Release { button: Button },
    /// One of the responses to a [device inquiry request](super::Output::request_device_inquiry)
    ApplicationVersion(Version),
    /// One of the responses to a [device inquiry request](super::Output::request_device_inquiry)
    BootloaderVersion(Version),
    /// The response to initialization commands to change the mode to Programmer mode
    ChangeLayout(Layout),
    /// The response to a [sleep mode request](super::Output::request_sleep_mode)
    SleepMode(SleepMode),
    /// The response to a [brigtness request](super::Output::request_brightness).
    Brightness(u8),
}

/// A version structure
///
/// The version is 4 bytes from 0-9.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Version {
    pub bytes: [u8; 4],
}

/// The Launchpad Mini MK3 input connection creator.
pub struct Input;

fn decode_grid_button(btn: u8) -> Button {
    let x = (btn % 10) - 1;
    let y = 8 - (btn / 10);
    Button::GridButton { x, y }
}

fn decode_control_button(btn: u8) -> Button {
    // The top control buttons are encoded as 91, 92, 95, 96, 97, 98, while the
    // right-side control buttons are encoded as 89, 79, 69, 59, 49, 39, 29, 19
    // (which fits in line with the grid button coordinates).
    //
    // In fact, Launchy considers the right-side control buttons as
    // grid buttons.
    match btn {
        91..=98 => Button::ControlButton { index: btn - 91 },
        19..=89 if btn % 10 == 9 => decode_grid_button(btn),
        _ => panic!("Unexpected control button value {}", btn),
    }
}

impl crate::InputDevice for Input {
    /// Device name.
    ///
    /// On MacOS, the Mini MK3 advertises
    ///
    /// - Launchpad Mini MK3 LPMiniMK3 DAW
    /// - Launchpad Mini MK3 LPMiniMK3 MIDI
    ///
    /// But only the MIDI interface works, so include the "MIDI" string.
    const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad Mini MK3 LPMiniMK3 MI";
    const MIDI_CONNECTION_NAME: &'static str = "Launchy Mini Mk3 Input";
    type Message = Message;

    fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
        match data {
            // Grid button
            &[0x90, button, velocity] => {
                let button = decode_grid_button(button);

                match velocity {
                    0 => Message::Release { button },
                    127 => Message::Press { button },
                    other => panic!("Unexpected grid note-on velocity {}", other),
                }
            }
            // Control button
            &[0xB0, number, velocity] => {
                let button = decode_control_button(number);

                match velocity {
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

                Message::Release { button }
            }
            // Response to a Device Inquiry
            &[240, 126, 0, 6, 2, 0, 32, 41, 19, 1, 0, 0, v1, v2, v3, v4, 247] => {
                Message::ApplicationVersion(Version {
                    bytes: [v1, v2, v3, v4],
                })
            }
            &[240, 126, 0, 6, 2, 0, 32, 41, 19, 17, 0, 0, v1, v2, v3, v4, 247] => {
                Message::BootloaderVersion(Version {
                    bytes: [v1, v2, v3, v4],
                })
            }
            // Response to sleep mode query
            &[240, 0, 32, 41, 2, 13, 9, sleep, 247] => Message::SleepMode(match sleep {
                0 => SleepMode::Sleep,
                _ => SleepMode::Wake,
            }),
            // Response to layout command
            &[240, 0, 32, 41, 2, 13, 14, layout, 247] => Message::ChangeLayout(layout.into()),
            // Response to brightness query
            &[240, 0, 32, 41, 2, 13, 8, brightness, 247] => Message::Brightness(brightness),
            other => panic!("Unexpected midi message: {:?}", other),
        }
    }
}
