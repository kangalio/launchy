pub(crate) mod double_buffering;
pub(crate) mod query;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
/// The button type used for Launchpads with 80 buttons
pub enum Button80 {
    ControlButton { index: u8 },
    GridButton { x: u8, y: u8 },
}

impl Button80 {
    pub const UP: Self = Button80::ControlButton { index: 0 };
    pub const DOWN: Self = Button80::ControlButton { index: 1 };
    pub const LEFT: Self = Button80::ControlButton { index: 2 };
    pub const RIGHT: Self = Button80::ControlButton { index: 3 };
    pub const SESSION: Self = Button80::ControlButton { index: 4 };

    /**
     * The 6th top-row button
     *
     * On the MK2 and the S, this button is called "User 1".
     *
     * On the MK3 Mini, this button is called "Drums".
     */
    pub const USER_1: Self = Button80::ControlButton { index: 5 };

    /**
     * The 7th top-row button
     *
     * On the MK2 and the S, this button is called "User 2".
     *
     * On the MK3 Mini, this button is called "Keys".
     */
    pub const USER_2: Self = Button80::ControlButton { index: 6 };

    /**
     * The 8th top-row button
     *
     * On the MK2 and the S, this button is called "Mixer".
     *
     * On the MK3 Mini, this button is called "User".
     */
    pub const MIXER: Self = Button80::ControlButton { index: 7 };
    pub const VOLUME: Self = Button80::GridButton { x: 8, y: 0 };
    pub const PAN: Self = Button80::GridButton { x: 8, y: 1 };
    pub const SEND_A: Self = Button80::GridButton { x: 8, y: 2 };
    pub const SEND_B: Self = Button80::GridButton { x: 8, y: 3 };
    pub const STOP: Self = Button80::GridButton { x: 8, y: 4 };
    pub const MUTE: Self = Button80::GridButton { x: 8, y: 5 };
    pub const SOLO: Self = Button80::GridButton { x: 8, y: 6 };
    pub const RECORD_ARM: Self = Button80::GridButton { x: 8, y: 7 };

    /// Creates a new GridButton coordinate
    pub fn grid(x: u8, y: u8) -> Button80 {
        Button80::GridButton { x, y }
    }

    /// Creates a new ControlButton coordinate
    pub fn control(index: u8) -> Button80 {
        Button80::ControlButton { index }
    }

    /// Creates a new button out of absolute coordinates, like the ones returned by `abs_x()` and
    /// `abs_y()`.
    pub fn from_abs(x: u8, y: u8) -> Button80 {
        match y {
            0 => {
                assert!(x <= 7);
                Button80::ControlButton { index: x }
            }
            1..=8 => {
                assert!(x <= 8);
                Button80::GridButton { x, y: y - 1 }
            }
            other => panic!("Unexpected y: {}", other),
        }
    }

    /// Returns x coordinate assuming coordinate origin in the leftmost control button
    pub fn abs_x(&self) -> u8 {
        match *self {
            Self::ControlButton { index } => index,
            Self::GridButton { x, .. } => x,
        }
    }

    /// Returns y coordinate assuming coordinate origin in the leftmost control button
    pub fn abs_y(&self) -> u8 {
        match *self {
            Self::ControlButton { .. } => 0,
            Self::GridButton { y, .. } => y + 1,
        }
    }

    /// Returns true if the button is part of the main 8x8 grid (absolute x: 0-7, absolute y: 1-8).
    pub fn is_main_grid_button(&self) -> bool {
        match self {
            Self::GridButton { x, y } => *x <= 7 && *y <= 7, // Internal y is 0-7
            _ => false,
        }
    }

    /// Returns true if the button is part of the top-row control buttons (absolute x: 0-7, absolute y: 0).
    pub fn is_top_control_button(&self) -> bool {
        match self {
            Self::ControlButton { index } => *index <= 7,
            _ => false,
        }
    }

    /// Returns true if the button is part of the right-column scene launch buttons (absolute x: 8, absolute y: 1-8).
    pub fn is_scene_launch_button(&self) -> bool {
        match self {
            Self::GridButton { x, y } => *x == 8 && *y <= 7, // Internal y is 0-7
            _ => false,
        }
    }
}
