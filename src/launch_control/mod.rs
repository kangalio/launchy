mod input;
pub use input::*;

mod output;
pub use output::*;


#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Template(u8);

impl Template {
	pub fn from_byte(byte: u8) -> Self {
		assert!(byte < 16);
		Self(byte)
	}
	
	pub fn user(index: u8) -> Self {
		assert!(index < 8);
		Self(index)
	}

	pub fn factory(index: u8) -> Self {
		assert!(index < 8);
		Self(index + 8)
	}
}

impl From<u8> for Template {
	fn from(index: u8) -> Self {
		return Template::from_byte(index);
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Button {
	Pad(u8),
	Up, Down, Left, Right
}

impl Button {
	pub fn from_index(index: u8) -> Self {
		match index {
			0..=7 => Self::pad(index),
			8..=11 => Self::control(index - 8),
			_ => panic!("Out of bounds index {}", index),
		}
	}

	pub fn as_index(self) -> u8 {
		match self {
			Self::Pad(index) => index,
			Self::Up => 8,
			Self::Down => 9,
			Self::Left => 10,
			Self::Right => 11,
		}
	}

	pub fn pad(index: u8) -> Self {
		assert!(index < 8);
		Self::Pad(index)
	}

	pub fn control(index: u8) -> Self {
		match index {
			0 => Self::Up,
			1 => Self::Down,
			2 => Self::Left,
			3 => Self::Right,
			_ => panic!("Out of bounds index {}", index),
		}
	}

	fn code(self) -> u8 {
		match self {
			Button::Pad(index @ 0..=3) => index + 9,
			Button::Pad(index @ 4..=7) => index + 21,
			Button::Pad(index) => panic!("Invalid pad index state (this is a bug) {}", index),
			Button::Up => 114,
			Button::Down => 115,
			Button::Left => 116,
			Button::Right => 117,
		}
	}
}

pub struct Spec;

impl crate::DeviceSpec for Spec {
    const BOUNDING_BOX_WIDTH: u32 = 10;
	const BOUNDING_BOX_HEIGHT: u32 = 2;
	const COLOR_PRECISION: u16 = 4;
	
    type Input = LaunchControlInput;
	type Output = LaunchControlOutput;
	
    fn is_valid(x: u32, y: u32) -> bool {
		if y == 0 && x <= 7 { return false }
		return true;
	}

	fn setup(output: &mut Self::Output) -> anyhow::Result<()> {
		output.change_template(0)
	}
	
    fn flush(
		canvas: &mut crate::DeviceCanvas<Self>,
		changes: &[(u32, u32, (u8, u8, u8))])
	-> anyhow::Result<()> {

        for &(x, y, (r, g, _b)) in changes {
			let button = match (x, y) {
				(8, 0) => Button::Up,
				(9, 0) => Button::Down,
				(8, 1) => Button::Left,
				(9, 1) => Button::Right,
				(index, 1) => Button::pad(index as u8),
				_ => panic!("Unexpected coordinates ({}|{})", x, y),
			};

			canvas.output.light(0, button, Color::new(r, g))?;
		}

		return Ok(());
    }
	
	fn convert_message(msg: Message) -> Option<crate::CanvasMessage> {
		fn button_to_xy(button: Button) -> (u32, u32) {
			match button {
				Button::Pad(index) => (index as u32, 1),
				Button::Up => (8, 0),
				Button::Down => (9, 0),
				Button::Left => (8, 1),
				Button::Right => (9, 1),
			}
		}

		match msg {
			Message::Press { template: _, button } => {
				let (x, y) = button_to_xy(button);
				Some(crate::CanvasMessage::Press { x, y })
			},
			Message::Release { template: _, button } => {
				let (x, y) = button_to_xy(button);
				Some(crate::CanvasMessage::Release { x, y })
			},
			_ => None,
		}
	}
}

pub type Canvas<'a> = crate::DeviceCanvas<'a, Spec>;