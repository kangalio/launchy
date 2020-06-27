mod input;
pub use input::*;

mod output;
pub use output::*;

// ---------- bla bla blah

use crate::CanvasMessage;

pub struct Canvas;

impl crate::DeviceCanvas for Canvas {
	type Input = LaunchpadSInput;
	type Output = LaunchpadSOutput;
	
	fn convert_message(msg: Message) -> Option<CanvasMessage> {
		match msg {
			Message::Press { button } => Some(CanvasMessage::Press {
				x: button.abs_x() as u32,
				y: button.abs_y() as u32,
			}),
			Message::Release { button } => Some(CanvasMessage::Release {
				x: button.abs_x() as u32,
				y: button.abs_y() as u32,
			}),
			_ => None,
		}
	}
}