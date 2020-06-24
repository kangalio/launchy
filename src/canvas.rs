use crate::Color;


pub trait Canvas {
	const BOUNDING_BOX_WIDTH: u32;
	const BOUNDING_BOX_HEIGHT: u32;

	// These are the methods that _need_ to be implemented by the.. implementor

	/// Check if the location is in bounds
	fn is_valid(x: u32, y: u32) -> bool;
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
		if !Self::is_valid(x, y) {
			panic!("Coordinates ({}|{}) out of bounds", x, y);
		}
		self.set_unchecked(x, y, color);
	}

	/// Sets the color at the given location. Panics if the location is out of bounds
	fn get(&self, x: u32, y: u32) -> Color {
		if !Self::is_valid(x, y) {
			panic!("Coordinates ({}|{}) out of bounds", x, y);
		}
		return self.get_unchecked(x, y);
	}

	/// Retrieves the old, unflushed color at the given location. Panics if the location is out of
	/// bounds
	fn get_old(&self, x: u32, y: u32) -> Color {
		if !Self::is_valid(x, y) {
			panic!("Coordinates ({}|{}) out of bounds", x, y);
		}
		return self.get_old_unchecked(x, y);
	}

	fn iter() -> CanvasIterator<Self> {
		return CanvasIterator::new();
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
	// These are on a valid state at the start, and right before the next valid state afterwards
	x: u32,
	y: u32,

	phantom: std::marker::PhantomData<C>, // dunno why rustc needs this but whatever
}

impl<C: Canvas + ?Sized> CanvasIterator<C> {
	fn new() -> Self {
		let mut iter = CanvasIterator {
			x: 0,
			y: 0,
			phantom: std::marker::PhantomData,
		};
		iter.find_next_valid(); // get to a valid state
		return iter;
	}

	fn advance(&mut self) {
		self.x += 1;
		if self.x == C::BOUNDING_BOX_WIDTH {
			self.x = 0;
			self.y += 1;
		}
	}

	// Returns false if there is no more valid state to go to
	fn find_next_valid(&mut self) -> bool {
		loop {
			if self.y >= C::BOUNDING_BOX_HEIGHT { return false }
			if C::is_valid(self.x, self.y) { return true }
			// if the current position is not out of bounds but still invalid, let's continue
			// searching
			self.advance();
		}
	}
}

impl<C: Canvas> Iterator for CanvasIterator<C> {
	type Item = CanvasButton<C>;

	fn next(&mut self) -> Option<Self::Item> {
		let in_bounds = self.find_next_valid();
		if !in_bounds { return None };

		let value = CanvasButton {
			x: self.x,
			y: self.y,
			phantom: std::marker::PhantomData,
		};

		self.advance();

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
	const BOUNDING_BOX_WIDTH: u32 = Backend::BOUNDING_BOX_WIDTH;
	const BOUNDING_BOX_HEIGHT: u32 = Backend::BOUNDING_BOX_HEIGHT;

	fn is_valid(x: u32, y: u32) -> bool { Backend::is_valid(x, y) }

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

		for y in 0..9 {
			for x in 0..9 {
				if !Self::is_valid(x, y) { continue }

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