pub(crate) mod double_buffering;
pub(crate) mod query;

/// The button type used for Launchpads with 80 buttons
///
/// This addresses the buttons by their function: the control buttons are
/// addressed by their index, and the grid buttons are addressed by their
/// coordinates.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum LogicalButton {
    ControlButton { index: u8 },
    GridButton { x: u8, y: u8 },
}

impl LogicalButton {
    pub const UP: Self = Self::ControlButton { index: 0 };
    pub const DOWN: Self = Self::ControlButton { index: 1 };
    pub const LEFT: Self = Self::ControlButton { index: 2 };
    pub const RIGHT: Self = Self::ControlButton { index: 3 };
    pub const SESSION: Self = Self::ControlButton { index: 4 };
    pub const USER_1: Self = Self::ControlButton { index: 5 };
    pub const USER_2: Self = Self::ControlButton { index: 6 };
    pub const MIXER: Self = Self::ControlButton { index: 7 };
    pub const VOLUME: Self = Self::GridButton { x: 8, y: 0 };
    pub const PAN: Self = Self::GridButton { x: 8, y: 1 };
    pub const SEND_A: Self = Self::GridButton { x: 8, y: 2 };
    pub const SEND_B: Self = Self::GridButton { x: 8, y: 3 };
    pub const STOP: Self = Self::GridButton { x: 8, y: 4 };
    pub const MUTE: Self = Self::GridButton { x: 8, y: 5 };
    pub const SOLO: Self = Self::GridButton { x: 8, y: 6 };
    pub const RECORD_ARM: Self = Self::GridButton { x: 8, y: 7 };

    /// Creates a new GridButton coordinate
    pub fn grid(x: u8, y: u8) -> Self {
        Self::GridButton { x, y }
    }

    /// Creates a new ControlButton coordinate
    pub fn control(index: u8) -> Self {
        Self::ControlButton { index }
    }
}

/// A physical button on a LaunchPad, addressed by its location on the pad
///
/// Physical buttons include control buttons. Not all physical locations are
/// necessarily occupied on any specific pad. For example, most 9x9 pads don't
/// have a button at (8, 0).
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct PhysicalButton {
    pub x: u32,
    pub y: u32,
}

impl PhysicalButton {
    pub fn new(x: u32, y: u32) -> Self {
        PhysicalButton { x, y }
    }
}
