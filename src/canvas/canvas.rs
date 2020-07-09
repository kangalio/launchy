use super::*;

/// A trait that abstracts over the specifics of a Launchpad and allows generic access and
/// manipulation of a Launchpad's LEDs.
/// 
/// # How do you use a canvas?
/// 
/// `Canvas`es work by first accumulating LED changes, and finally flushing all LED state changes
/// in an efficient manner by calling `.flush()`.
/// 
/// Every `Canvas` maintains two buffers: the unflushed one, and the edited one. Therefore, you
/// can access both the unflushed and the buffered state of the button, using `get_old` and `get`,
/// respectively.
/// 
/// Example:
/// ```rust
/// fn light_white(canvas: &mut impl Canvas) -> Result<()> {
/// 	// Iterate through all buttons in the canvas. See the documentation on `CanvasIterator` for
/// 	// more info
/// 	for button in canvas.iter() {
/// 		button.set(canvas, Color::WHITE);
/// 	}
/// }
/// 
/// // The above function doesn't take a specific low-level object like Output or
/// // Output. Instead it utilizes Canvas, so you can call it with _all_ devices!
/// 
/// // Light a connected Launchpad S and Launchpad Mk2 completely white
/// light_white(&mut launchy::s::Canvas::guess());
/// light_white(&mut launchy::mk2::Canvas::guess());
/// ```
pub trait Canvas: std::ops::Index<Pad, Output=Color> + std::ops::IndexMut<Pad, Output=Color> {
	// These are the methods that _need_ to be implemented by the implementor

	/// The width of the smallest rectangle that still fully encapsulates the shape of this canvas
	fn bounding_box_width(&self) -> u32;
	/// The height of the smallest rectangle that still fully encapsulates the shape of this canvas
	fn bounding_box_height(&self) -> u32;
	/// Check if the location is in bounds
	fn is_valid(&self, x: u32, y: u32) -> bool;
	
	/// Returns a reference to the color at the given position. No bounds checking
	fn get_old_unchecked_ref(&self, x: u32, y: u32) -> &Color;
	/// Returns a reference to the in-buffer/unflushed color at the given position. No bounds
	/// checking
	fn get_new_unchecked_ref(&self, x: u32, y: u32) -> &Color;
	/// Returns a mutable reference to the color at the given position. No bounds checking
	fn get_new_unchecked_mut(&mut self, x: u32, y: u32) -> &mut Color;
	
	/// Flush the accumulated changes to the underlying device
	fn flush(&mut self) -> Result<(), crate::MidiError>;
	/// The lowest visible brightness on this canvas. Used to calibrate brightness across Launchpads
	fn lowest_visible_brightness(&self) -> f32;
	

	// These are defaut implementations that you get for free

	// Returns the color at the given position, or None if out of bounds
	fn get(&self, pad: Pad) -> Option<Color> {
		if pad.x >= 0 && pad.y >= 0 && self.is_valid(pad.x as u32, pad.y as u32) {
			Some(*self.get_old_unchecked_ref(pad.x as u32, pad.y as u32))
		} else {
			None
		}
	}

	// Returns the color at the given position. No bounds checking
	fn get_old_unchecked(&self, x: u32, y: u32) -> Color {
		*self.get_old_unchecked_ref(x, y)
	}

	// Returns the in-buffer/unflushed color at the given position. No bounds checking
	fn get_new_unchecked(&self, x: u32, y: u32) -> Color {
		*self.get_new_unchecked_ref(x, y)
	}

	// Set the color at the given position. No bounds checking
	fn set_unchecked(&mut self, x: u32, y: u32, color: Color) {
		*self.get_new_unchecked_mut(x, y) = color;
	}

	// Returns a reference to the color at the given position, or None if out of bounds
	fn get_ref(&self, pad: Pad) -> Option<&Color> {
		if pad.x >= 0 && pad.y >= 0 && self.is_valid(pad.x as u32, pad.y as u32) {
			Some(self.get_old_unchecked_ref(pad.x as u32, pad.y as u32))
		} else {
			None
		}
	}

	// Returns a mutable reference to the color at the given position, or None if out of bounds
	fn get_mut(&mut self, pad: Pad) -> Option<&mut Color> {
		if pad.x >= 0 && pad.y >= 0 && self.is_valid(pad.x as u32, pad.y as u32) {
			Some(self.get_new_unchecked_mut(pad.x as u32, pad.y as u32))
		} else {
			None
		}
	}

	/// Returns the old, unflushed color at the given location, or None if out of bounds
	fn get_new(&self, pad: Pad) -> Option<Color> {
		if pad.x >= 0 && pad.y >= 0 && self.is_valid(pad.x as u32, pad.y as u32) {
			Some(*self.get_new_unchecked_ref(pad.x as u32, pad.y as u32))
		} else {
			None
		}
	}

	/// Returns the old, unflushed color at the given location. Panics if out of bounds
	fn at_new(&self, pad: Pad) -> Color {
		self.get_new(pad).expect("Pad coordinates out of bounds")
	}

	// Sets the color at the given position. Returns None if out of bounds
	#[must_use]
	fn set(&mut self, pad: Pad, color: Color) -> Option<()> {
		*self.get_mut(pad)? = color;
		Some(())
	}

	/// An iterator over the buttons of a given Canvas. Create an iterator by calling `.iter()` on a
	/// `Canvas`.
	/// 
	/// This iterator returns `CanvasButton`s, which are a view on a single button on the canvas. See
	/// the documentation on `CanvasButton` for more information.
	/// 
	/// For example to light the entire canvas white:
	/// ```rust
	/// for button in canvas.iter() {
	/// 	button.set(&mut canvas, Color::WHITE);
	/// }
	/// 
	/// canvas.flush();
	/// ```
	/// 
	/// Or, if you want to move the entire contents of the canvas one pixel to the right:
	/// ```rust
	/// for button in canvas.iter() {
	/// 	let (x, y) = (button.x(), button.y());
	/// 	if canvas.is_valid(x - 1, y) { // if there is a pixel to the left of this one
	/// 		// Get the unflushed color from the left pixel and move it to this pixel
	/// 		canvas.set(x, y, canvas.get_old(x - 1, y))
	/// 	}
	/// }
	/// 
	/// canvas.flush();
	/// ```
	fn iter(&self) -> CanvasIterator {
		return CanvasIterator::new(self);
	}

	/// Toggles the button at the specified coordinate with the given color.
	/// 
	/// Light the specified button with the given color, except if the button is already lit with
	/// that color; in which case the button is turned off.
	/// 
	/// For example, if you were to make a paint program for the Launchpad MK2 where you can toggle
	/// pixels by pressing:
	/// ```rust
	/// let mut (canvas, poller) = launchy::mk2::Canvas::guess();
	/// 
	/// for msg in poller.iter() {
	/// 	if let CanvasMessage::Press { x, y } = msg {
	/// 		canvas.toggle(x, y, Color::WHITE);
	/// 	}
	/// }
	/// ```
	fn toggle(&mut self, pad: Pad, color: Color) -> Option<()> {
		if self.get(pad)? == color {
			self.set(pad, Color::BLACK)?;
		} else {
			self.set(pad, color)?;
		}
		Some(())
	}

	/// Clear the entire canvas by setting all buttons to black.
	/// 
	/// ```rust
	/// // light a square in the top left, for one second
	/// 
	/// canvas.set(0, 0, Color::MAGENTA);
	/// canvas.set(0, 1, Color::MAGENTA);
	/// canvas.set(1, 0, Color::MAGENTA);
	/// canvas.set(1, 1, Color::MAGENTA);
	/// 
	/// std::thread::sleep_ms(1000);
	/// 
	/// canvas.clear();
	/// ```
	fn clear(&mut self) where Self: Sized {
		for pad in self.iter() {
			self[pad] = Color::BLACK;
		}
	}

	fn into_padded(self) -> PaddingCanvas<Self> where Self: Sized {
		PaddingCanvas::from(self)
	}
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum CanvasMessage {
	Press { x: u32, y: u32 },
	Release { x: u32, y: u32 },
}

impl CanvasMessage {
	/// Retrieves the x coordinate of this message, no matter if this is a press or a release
	/// message
	pub fn x(&self) -> u32 {
		match *self {
			Self::Press { x, y: _ } => x,
			Self::Release { x, y: _ } => x,
		}
	}

	/// Retrieves the y coordinate of this message, no matter if this is a press or a release
	/// message
	pub fn y(&self) -> u32 {
		match *self {
			Self::Press { x: _, y } => y,
			Self::Release { x: _, y } => y,
		}
	}

	pub fn pad(&self) -> Pad {
		Pad { x: self.x() as i32, y: self.y() as i32 }
	}

	/// Returns whether this is a press message
	pub fn is_press(&self) -> bool {
		match self {
			Self::Press { .. } => true,
			_ => false,
		}
	}

	/// Returns whether this is a release message
	pub fn is_release(&self) -> bool {
		match self {
			Self::Release { .. } => true,
			_ => false,
		}
	}
}