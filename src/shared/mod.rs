//! Routines that are shared between the different LaunchPad implementations

use crate::{
    protocols::{LogicalButton, PhysicalButton},
    DeviceSpec,
};

/// Default implementation of [super::DeviceSpec::to_logical].
///
/// This implementation is appropriate for LaunchControl, Mini, MK2, and S.
///
/// It uses [DeviceSpec::is_valid] to determine whether or not to return `None`.
///
/// # Panics
///
/// If the DeviceSpec returns `true` but the coordinates are out of range of a `9x9` grid.
pub fn default_physical_to_logical<D: DeviceSpec>(button: PhysicalButton) -> Option<LogicalButton> {
    if !D::is_valid(button.x, button.y) {
        return None;
    }

    match button.y {
        0 => {
            if button.x < 7 {
                Some(LogicalButton::ControlButton {
                    index: button.x as u8,
                })
            } else {
                panic!("Control Button the LaunchPad indicates is valid is too high: {}; do not call default_physical_to_logical", button.x)
            }
        }
        1..=8 => {
            if button.x <= 8 {
                Some(LogicalButton::GridButton {
                    x: button.x as u8,
                    y: button.y as u8 - 1,
                })
            } else {
                panic!("Grid Button the LaunchPad indicates is valid is too high: ({}, {}); do not call default_physical_to_logical", button.x, button.y)
            }
        }
        _ => None,
    }
}

/// Default implementation of [super::DeviceSpec::to_physical].
///
/// This implementation is appropriate for LaunchControl, Mini, MK2, and S.
pub fn default_logical_to_physical(button: LogicalButton) -> PhysicalButton {
    match button {
        LogicalButton::ControlButton { index } => PhysicalButton {
            x: index as u32,
            y: 0,
        },
        LogicalButton::GridButton { x, y } => PhysicalButton {
            x: x as u32,
            y: y as u32 + 1,
        },
    }
}
