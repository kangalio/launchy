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