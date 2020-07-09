#[macro_export]
macro_rules! ok_or_continue {
	( $e:expr ) => (
		match $e {
			Ok(value) => value,
			Err(_e) => {
				continue;
			},
		}
	)
}

#[macro_export]
macro_rules! some_or_continue {
	( $e:expr ) => (
		match $e {
			Some(value) => value,
			None => {
				continue
			},
		}
	)
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Array2d<T: Default + Copy> {
	width: usize,
	height: usize,
	vec: Vec<T>,
}

impl<T: Default + Copy> Array2d<T> {
	pub fn new(width: usize, height: usize) -> Self {
		Self { width, height, vec: vec![T::default(); width * height] }
	}

	pub fn is_valid(&self, x: usize, y: usize) -> bool {
		x < self.width && y < self.height
	}

	pub fn get(&self, x: usize, y: usize) -> T {
		assert!(self.is_valid(x, y));

		self.vec[y * self.width + x]
	}

	pub fn get_ref(&self, x: usize, y: usize) -> &T {
		assert!(self.is_valid(x, y));

		&self.vec[y * self.width + x]
	}

	pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
		assert!(self.is_valid(x, y));

		&mut self.vec[y * self.width + x]
	}

	pub fn set(&mut self, x: usize, y: usize, value: T) {
		*self.get_mut(x, y) = value;
	}

	pub fn width(&self) -> usize { self.width }
	pub fn height(&self) -> usize { self.height }
}