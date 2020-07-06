use super::*;


#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Hash)]
pub struct Pad {
	pub x: i32,
	pub y: i32,
}

impl Pad {
	pub fn up(self, steps: i32) -> Self {
		Self { x: self.x, y: self.y - steps }
	}

	pub fn down(self, steps: i32) -> Self {
		Self { x: self.x, y: self.y + steps }
	}

	pub fn left(self, steps: i32) -> Self {
		Self { x: self.x - steps, y: self.y }
	}

	pub fn right(self, steps: i32) -> Self {
		Self { x: self.x + steps, y: self.y }
	}

	pub fn neighbors_4(self) -> [Self; 4] {
		[
			self.up(1),
			self.right(1),
			self.down(1),
			self.left(1),
		]
	}

	pub fn neighbors_5(self) -> [Self; 5] {
		[
			self,
			self.up(1),
			self.right(1),
			self.down(1),
			self.left(1),
		]
	}

	pub fn neighbors_8(self) -> [Self; 8] {
		[
			self.up(1),
			self.up(1).right(1),
			self.right(1),
			self.right(1).down(1),
			self.down(1),
			self.down(1).left(1),
			self.left(1),
			self.left(1).up(1),
		]
	}

	pub fn neighbors_9(self) -> [Self; 9] {
		[
			self,
			self.up(1),
			self.up(1).right(1),
			self.right(1),
			self.right(1).down(1),
			self.down(1),
			self.down(1).left(1),
			self.left(1),
			self.left(1).up(1),
		]
	}

	/// Get the color of this button in the given canvas
    pub fn get(self, canvas: &impl Canvas) -> Color {
		canvas.get(self).expect("Coordinates out of bounds")
	}

	/// Get the unflushed color of this button in the given canvas
    pub fn get_old(self, canvas: &impl Canvas) -> Color {
		canvas.get_old(self).expect("Coordinates out of bounds")
	}

	/// Set the color of this button in the given canvas
	pub fn set(self, canvas: &mut impl Canvas, color: Color) {
		canvas.set(self, color).expect("Coordinates out of bounds");
	}
}

impl std::ops::Add<(i32, i32)> for Pad {
	type Output = Self;

	fn add(self, offset: (i32, i32)) -> Self {
		let (x_offset, y_offset) = offset;
		Self {
			x: self.x + x_offset,
			y: self.y + y_offset,
		}
	}
}

impl std::ops::AddAssign<(i32, i32)> for Pad {
	fn add_assign(&mut self, offset: (i32, i32)) {
		let (x_offset, y_offset) = offset;
		self.x += x_offset;
		self.y += y_offset;
	}
}

impl std::ops::Sub<(i32, i32)> for Pad {
	type Output = Self;

	fn sub(self, offset: (i32, i32)) -> Self {
		let (x_offset, y_offset) = offset;
		Self {
			x: self.x - x_offset,
			y: self.y - y_offset,
		}
	}
}

impl std::ops::SubAssign<(i32, i32)> for Pad {
	fn sub_assign(&mut self, offset: (i32, i32)) {
		let (x_offset, y_offset) = offset;
		self.x -= x_offset;
		self.y -= y_offset;
	}
}