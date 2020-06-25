use crate::Color;


pub trait Canvas {
	// These are the methods that _need_ to be implemented by the.. implementor

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

	/// Sets the color at the given location. Panics if the location is out of bounds
	fn set(&mut self, x: u32, y: u32, color: Color) {
		if !self.is_valid(x, y) {
			panic!("Coordinates ({}|{}) out of bounds", x, y);
		}
		self.set_unchecked(x, y, color);
	}

	/// Sets the color at the given location. Panics if the location is out of bounds
	fn get(&self, x: u32, y: u32) -> Color {
		if !self.is_valid(x, y) {
			panic!("Coordinates ({}|{}) out of bounds", x, y);
		}
		return self.get_unchecked(x, y);
	}

	/// Retrieves the old, unflushed color at the given location. Panics if the location is out of
	/// bounds
	fn get_old(&self, x: u32, y: u32) -> Color {
		if !self.is_valid(x, y) {
			panic!("Coordinates ({}|{}) out of bounds", x, y);
		}
		return self.get_old_unchecked(x, y);
	}

	fn iter(&self) -> CanvasIterator<Self> {
		return CanvasIterator::new(self);
	}

	// fn iter_mut(&mut self) -> CanvasIteratorMut<Self> {
	// 	return CanvasIteratorMut::new(self);
	// }
}

// Next lines are canvas iteration stuff...

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CanvasButton<C: Canvas + ?Sized> {
	// canvas button coordinates MUST be valid!
	x: u32,
	y: u32,

	// we need to restrict ourselves to just this specific canvas type, because the coordinates may
	// be invalid for other canvas types
	phantom: std::marker::PhantomData<C>,
}

impl<C: Canvas + ?Sized> CanvasButton<C> {
	pub fn x(&self) -> u32 { self.x }
	pub fn y(&self) -> u32 { self.y }

    pub fn get(&self, canvas: &C) -> Color {
		canvas.get_unchecked(self.x, self.y)
	}

    pub fn get_old(&self, canvas: &C) -> Color {
		canvas.get_old_unchecked(self.x, self.y)
	}

	pub fn set(&self, canvas: &mut C, color: Color) {
		canvas.set_unchecked(self.x, self.y, color);
	}
}

pub struct CanvasIterator<C: Canvas + ?Sized> {
	coordinates: Vec<(u32, u32)>, // the list of coordinates that we will iterate through
	index: usize,
	phantom: std::marker::PhantomData<C>, // dunno why rustc needs this but whatever
}

impl<C: Canvas + ?Sized> CanvasIterator<C> {
	fn new(canvas: &C) -> Self {
		let bb_height = canvas.bounding_box_height();
		let bb_width = canvas.bounding_box_width();

		let mut coordinates = Vec::with_capacity((bb_width * bb_height) as usize);
		for y in 0..bb_height {
			for x in 0..bb_width {
				if canvas.is_valid(x, y) {
					coordinates.push((x, y));
				}
			}
		}

		return CanvasIterator {
			coordinates,
			index: 0,
			phantom: std::marker::PhantomData,
		};
	}
}

impl<C: Canvas> Iterator for CanvasIterator<C> {
	type Item = CanvasButton<C>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index >= self.coordinates.len() {
			return None;
		}

		let value = CanvasButton {
			x: self.coordinates[self.index].0,
			y: self.coordinates[self.index].1,
			phantom: std::marker::PhantomData,
		};

		self.index += 1;

		return Some(value);
	}
}

// now we get to the generic canvas stuff...

pub trait Flushable {
	const BOUNDING_BOX_WIDTH: u32;
	const BOUNDING_BOX_HEIGHT: u32;

	fn is_valid(x: u32, y: u32) -> bool;
	fn flush(&mut self, changes: &[(u32, u32, crate::Color)]) -> anyhow::Result<()>;
}

pub struct GenericCanvas<Backend: Flushable> {
	pub backend: Backend,
	curr_state: crate::util::Array2d<crate::Color>,
	new_state: crate::util::Array2d<crate::Color>,
}

impl<Backend: Flushable> GenericCanvas<Backend> {
	/// The passed-in backend must not have been used already. The canvas relies on a 'blank state',
	/// so to say.
	pub fn new(backend: Backend) -> Self {
		let curr_state = crate::util::Array2d::new(9, 9);
		let new_state = crate::util::Array2d::new(9, 9);
		return Self { backend, curr_state, new_state };
	}
}

impl<Backend: Flushable> crate::Canvas for GenericCanvas<Backend> {
	fn bounding_box_width(&self) -> u32 { Backend::BOUNDING_BOX_WIDTH }
	fn bounding_box_height(&self) -> u32 { Backend::BOUNDING_BOX_HEIGHT }
	fn is_valid(&self, x: u32, y: u32) -> bool { Backend::is_valid(x, y) }

	fn set_unchecked(&mut self, x: u32, y: u32, color: crate::Color) {
		self.new_state.set(x as usize, y as usize, color);
	}

	fn get_unchecked(&self, x: u32, y: u32) -> crate::Color {
		return self.new_state.get(x as usize, y as usize);
	}

	fn get_old_unchecked(&self, x: u32, y: u32) -> crate::Color {
		return self.curr_state.get(x as usize, y as usize);
	}

	fn flush(&mut self) -> anyhow::Result<()> {
		let mut changes: Vec<(u32, u32, crate::Color)> = Vec::with_capacity(9 * 9);

		// TODO: use iterator here
		for y in 0..9 {
			for x in 0..9 {
				if !self.is_valid(x, y) { continue }

				if self.get(x, y) != self.get_old(x, y) {
					let color = self.get(x, y);
					changes.push((x, y, color));
				}
			}
		}

		if changes.len() > 0 {
			self.backend.flush(&changes)?;
		}

		self.curr_state = self.new_state.clone();

		return Ok(());
	}
}