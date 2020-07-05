use super::*;
use crate::Color;


/// A view on a single button in a canvas. Allows retrieving and manipulating the color at this
/// position.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CanvasButton {
	// canvas button coordinates MUST be valid!
	x: u32,
	y: u32,
}

impl CanvasButton {
	/// The x coordinate of this button
	pub fn x(&self) -> u32 { self.x }
	/// The y coordinate of this button
	pub fn y(&self) -> u32 { self.y }

	/// Get the color of this button in the given canvas
    pub fn get(&self, canvas: &impl Canvas) -> Color {
		canvas.get_unchecked(self.x, self.y)
	}

	/// Get the unflushed color of this button in the given canvas
    pub fn get_old(&self, canvas: &impl Canvas) -> Color {
		canvas.get_old_unchecked(self.x, self.y)
	}

	/// Set the color of this button in the given canvas
	pub fn set(&self, canvas: &mut impl Canvas, color: Color) {
		canvas.set_unchecked(self.x, self.y, color);
	}
}

/// An iterator over the buttons of a given Canvas. Create an iterator by calling `.iter()` on a
/// `Canvas`.
/// 
/// For more information, see `Canvas::iter()`.
pub struct CanvasIterator(std::vec::IntoIter<CanvasButton>);

impl CanvasIterator {
	pub(crate) fn new<C: Canvas + ?Sized>(canvas: &C) -> Self {
		let bb_height = canvas.bounding_box_height();
		let bb_width = canvas.bounding_box_width();

		let mut coordinates = Vec::with_capacity((bb_width * bb_height) as usize);
		for y in 0..bb_height {
			for x in 0..bb_width {
				if canvas.is_valid(x, y) {
					coordinates.push(CanvasButton { x, y });
				}
			}
		}

		return CanvasIterator(coordinates.into_iter());
	}
}

impl Iterator for CanvasIterator {
	type Item = CanvasButton;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}