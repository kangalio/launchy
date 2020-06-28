mod input;
pub use input::*;

mod output;
pub use output::*;

pub use crate::protocols::Button80 as Button;

pub struct Spec;

impl crate::DeviceSpec for Spec {
    const BOUNDING_BOX_WIDTH: u32 = 9;
	const BOUNDING_BOX_HEIGHT: u32 = 9;
	
    type Input = LaunchpadMk2Input;
	type Output = LaunchpadMk2Output;
	
    fn is_valid(x: u32, y: u32) -> bool {
        if x > 8 || y > 8 { return false }
		if x == 8 && y == 0 { return false }
		return true;
	}
	
    fn flush(output: &mut Self::Output, changes: &[(u32, u32, crate::Color)]) -> anyhow::Result<()> {
        let changes = changes.iter().map(|&(x, y, color)| {
			let (r, g, b) = color.quantize_human(64);
			let color = RgbColor::new(r, g, b);

			let button = Button::from_abs(x as u8, y as u8);

			return (button, color);
		});
		return output.light_multiple_rgb(changes);
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