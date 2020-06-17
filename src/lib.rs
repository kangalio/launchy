#![allow(unused_imports)]

mod util;

pub mod launchpad_s;
pub use launchpad_s as s;

pub mod launchpad_mk2;
pub use launchpad_mk2 as mk2;


/// Identifier used for e.g. the midi port names etc.
const APPLICATION_NAME: &'static str = "LaunchpadRs";


#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Button {
	ControlButton { number: u8 },
	GridButton { x: u8, y: u8 },
}

impl Button {
	/// Returns x coordinate assuming coordinate origin in the leftmost control button
	pub fn abs_x(&self) -> u8 {
		match self {
			Self::ControlButton { number } => return *number,
			Self::GridButton { x, .. } => return *x,
		}
	}

	/// Returns y coordinate assuming coordinate origin in the leftmost control button
	pub fn abs_y(&self) -> u8 {
		match self {
			Self::ControlButton { .. } => return 0,
			Self::GridButton { y, .. } => y + 1,
		}
	}
}