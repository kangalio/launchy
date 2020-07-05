// I know explicitly returning at the end of functions is not idiomatic, but I prefer it personally.
// Also, I use tabs everywhere and I don't agree with clippy's reasoning against tabs in doc
// comments, so I will keep using tabs in doc comments
#![allow(clippy::needless_return, clippy::tabs_in_doc_comments)]

pub mod util;

mod protocols;

pub mod canvas;
pub use canvas::*;

mod midi_io;
pub use midi_io::*;

pub mod launchpad_s;
pub use launchpad_s as s;

pub mod launchpad_mk2;
pub use launchpad_mk2 as mk2;

pub mod launch_control;
pub use launch_control as control;

pub mod prelude {
	pub use crate::midi_io::{OutputDevice, InputDevice, MsgPollingWrapper};
	pub use crate::canvas::{Canvas, Color};
}

/// Identifier used for e.g. the midi port names etc.
const APPLICATION_NAME: &str = "LaunchpadRs";