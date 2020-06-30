mod input;
pub use input::*;

mod output;
pub use output::*;

pub use crate::protocols::Button80 as Button;


pub struct Spec;

impl crate::DeviceSpec for Spec {
    const BOUNDING_BOX_WIDTH: u32 = 9;
	const BOUNDING_BOX_HEIGHT: u32 = 9;
	const COLOR_PRECISION: u8 = 4;
	
    type Input = LaunchpadSInput;
	type Output = LaunchpadSOutput;
	
    fn is_valid(x: u32, y: u32) -> bool {
        if x > 8 || y > 8 { return false }
		if x == 8 && y == 0 { return false }
		return true;
	}
	
    fn flush(output: &mut Self::Output, changes: &[(u32, u32, (u8, u8, u8))]) -> anyhow::Result<()> {
        for &(x, y, (r, g, _b)) in changes {
			output.light(Button::from_abs(x as u8, y as u8), Color::new(r, g))?;
		}

		return Ok(());
    }
	
	fn convert_message(msg: Message) -> Option<crate::CanvasMessage> {
		match msg {
			Message::Press { button } => Some(crate::CanvasMessage::Press {
				x: button.abs_x() as u32,
				y: button.abs_y() as u32,
			}),
			Message::Release { button } => Some(crate::CanvasMessage::Release {
				x: button.abs_x() as u32,
				y: button.abs_y() as u32,
			}),
			_ => None,
		}
	}
}

pub type Canvas<'a> = crate::DeviceCanvas<'a, Spec>;