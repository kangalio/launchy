#![allow(unused_imports)]

mod util;

pub mod launchpad_s;
pub use launchpad_s as s;

pub mod launchpad_mk2;
pub use launchpad_mk2 as mk2;


/// Identifier used for e.g. the midi port names etc.
const APPLICATION_NAME: &'static str = "LaunchpadRs";


#[derive(Debug)]
pub enum Button {
	ControlButton { number: u8 },
	GridButton { x: u8, y: u8 },
}