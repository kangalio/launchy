// WOW Clippy HATES being explicit
// and also, the reasoning against tabs in doc comments is exactly the same reasoning against tabs
// as indentation in general - and that is totally stupid, because indentation style is something
// subjective. Guess clippy missed the note about that.
#![allow(clippy::needless_return, clippy::tabs_in_doc_comments)]

mod util;

mod protocols;

mod color;
pub use color::*;

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
	pub use crate::canvas::Canvas;
}

/// Identifier used for e.g. the midi port names etc.
const APPLICATION_NAME: &str = "LaunchpadRs";