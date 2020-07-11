/// A 2-bit color, with only red and green components
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub struct Color {
	red: u8,
	green: u8,
}

impl Color {
	pub const BLACK: Color = Color { red: 0, green: 0 };
	pub const RED: Color = Color { red: 3, green: 0 };
	pub const GREEN: Color = Color { red: 0, green: 3 };
	pub const YELLOW: Color = Color { red: 3, green: 3 };

	/// Create a new color from the given red and green components.
	/// 
	/// Both values must be less than 4 (they are 2-bit values)
	pub fn new(red: u8, green: u8) -> Color {
		assert!(red < 4);
		assert!(green < 4);

		return Color { red, green };
	}

	pub fn red(&self) -> u8 { self.red }
	pub fn green(&self) -> u8 { self.green }
	pub fn set_red(&mut self, red: u8) { assert!(red < 4); self.red = red }
	pub fn set_green(&mut self, green: u8) { assert!(green < 4); self.green = green }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Brightness {
	Off, Low, Medium, Full
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
#[repr(u8)]
pub enum Buffer {
	A = 0,
	B = 1,
}

/// This enum specifies how a light state change should affect the other buffer, if at all
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum DoubleBufferingBehavior {
	/// Only write to the currently edited buffer
	None,
	/// Clear the other buffer's copy of this LED
	Clear,
	/// Write this LED data to both buffers
	Copy,
}

/// Specifies a double buffering mode change
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct DoubleBuffering {
	// If true, copy the contents from the new "displayed" buffer to the new "edited" buffer
	pub copy: bool,
	// If true, continually flip "displayed" buffers to make a flashing effect
	pub flash: bool,
	// The buffer that is being edited
	pub edited_buffer: Buffer,
	// The buffer that is being displayed
	pub displayed_buffer: Buffer,
}

pub(crate) fn make_color_code(color: Color, dbb: DoubleBufferingBehavior)
		-> u8 {
	
	// Bit 6 - Must be 0
	// Bit 5..4 - Green LED brightness
	// Bit 3 - Clear - If 1: clear the other buffer’s copy of this LED.
	// Bit 2 - Copy - If 1: write this LED data to both buffers.
	// Bit 1..0 - Red LED brightness
	let double_buffering_code = match dbb {
		DoubleBufferingBehavior::None => 0b00,
		DoubleBufferingBehavior::Copy => 0b01,
		DoubleBufferingBehavior::Clear => 0b10,
	};
	return (color.green << 4) | (double_buffering_code << 2) | color.red;
}