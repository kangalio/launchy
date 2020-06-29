use super::*;


/// A trait that abstracts over the specifics of a Launchpad and allows generic access and
/// manipulation of a Launchpad's LEDs.
/// 
/// **How do you use a canvas?**
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
/// // The above function doesn't take a specific low-level object like LaunchpadMk2Output or
/// // LaunchControlOutput. Instead it utilizes Canvas, so you can call it with _all_ devices!
/// 
/// // Light a connected Launchpad S and Launchpad Mk2 completely white
/// light_white(&mut launchy::s::Canvas::guess());
/// light_white(&mut launchy::mk2::Canvas::guess());
/// ```
pub trait Canvas {
	// These are the methods that _need_ to be implemented by the implementor

	/// The width of the smallest rectangle that still fully encapsulates the shape of this canvas
	fn bounding_box_width(&self) -> u32;
	/// The height of the smallest rectangle that still fully encapsulates the shape of this canvas
	fn bounding_box_height(&self) -> u32;
	/// Check if the location is in bounds
	fn is_valid(&self, x: u32, y: u32) -> bool;
	/// Retrieves the current color at the given location. No bounds checking
	fn get_unchecked(&self, x: u32, y: u32) -> Color;
	/// Sets the color at the given location. No bounds checking
	fn set_unchecked(&mut self, x: u32, y: u32, color: Color);
	/// Retrieves the old, unflushed color at the given location. No bounds checking
	fn get_old_unchecked(&self, x: u32, y: u32) -> Color;
	/// Flush the accumulated changes to the underlying device
	fn flush(&mut self) -> anyhow::Result<()>;
	
	// These are defaut implementations that you get for free

	/// Sets the color at the given location. Returns None if the location is out of bounds
	fn set(&mut self, x: u32, y: u32, color: Color) -> Option<()> {
		if self.is_valid(x, y) {
			self.set_unchecked(x, y, color);
			Some(())
		} else {
			None
		}
	}

	/// Sets the color at the given location. Returns None if the location is out of bounds
	fn get(&self, x: u32, y: u32) -> Option<Color> {
		if self.is_valid(x, y) {
			Some(self.get_unchecked(x, y))
		} else {
			None
		}
	}

	/// Retrieves the old, unflushed color at the given location. Returns None if the location is
	/// out of bounds
	fn get_old(&self, x: u32, y: u32) -> Option<Color> {
		if self.is_valid(x, y) {
			Some(self.get_old_unchecked(x, y))
		} else {
			None
		}
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
	fn toggle(&mut self, x: u32, y: u32, color: Color) -> Option<()> {
		if self.get(x, y)? == color {
			self.set(x, y, Color::BLACK)?;
		} else {
			self.set(x, y, color)?;
		}
		Some(())
	}
}

/// A message from a `Canvas`.
/// 
/// Example:
/// ```rust
/// let _canvas = launchy::mk2::Canvas::guess(|msg| {
/// 	match msg {
/// 		CanvasMessage::Press { x, y } => println!("Pressed button at ({}|{})", x, y);
/// 		CanvasMessage::Release { x, y } => println!("Released button at ({}|{})", x, y);
/// 	}
/// });
/// ```
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