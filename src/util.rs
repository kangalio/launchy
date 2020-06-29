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
		return Self { width, height, vec: vec![T::default(); width * height] };
	}

	pub fn get(&self, x: usize, y: usize) -> T {
		assert!(x < self.width);
		assert!(y < self.height);

		return self.vec[y * self.width + x];
	}

	pub fn set(&mut self, x: usize, y: usize, value: T) {
		assert!(x < self.width);
		assert!(y < self.height);

		self.vec[y * self.width + x] = value;
	}

	pub fn width(&self) -> usize { self.width }
	pub fn height(&self) -> usize { self.height }
}