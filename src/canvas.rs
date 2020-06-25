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

	fn iter(&self) -> CanvasIterator {
		return CanvasIterator::new(self);
	}
}

pub trait IntoCanvas {
	type CanvasType: Canvas;

	fn into_canvas(self) -> Self::CanvasType;
}

// Next lines are canvas iteration stuff...

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CanvasButton {
	// canvas button coordinates MUST be valid!
	x: u32,
	y: u32,
}

impl CanvasButton {
	pub fn x(&self) -> u32 { self.x }
	pub fn y(&self) -> u32 { self.y }

    pub fn get(&self, canvas: &impl Canvas) -> Color {
		canvas.get_unchecked(self.x, self.y)
	}

    pub fn get_old(&self, canvas: &impl Canvas) -> Color {
		canvas.get_old_unchecked(self.x, self.y)
	}

	pub fn set(&self, canvas: &mut impl Canvas, color: Color) {
		canvas.set_unchecked(self.x, self.y, color);
	}
}

pub struct CanvasIterator {
	coordinates: Vec<(u32, u32)>, // the list of coordinates that we will iterate through
	index: usize,
}

impl CanvasIterator {
	fn new<C: Canvas + ?Sized>(canvas: &C) -> Self {
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
		};
	}
}

impl Iterator for CanvasIterator {
	type Item = CanvasButton;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index >= self.coordinates.len() {
			return None;
		}

		let value = CanvasButton {
			x: self.coordinates[self.index].0,
			y: self.coordinates[self.index].1,
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

impl<T> IntoCanvas for T where T: Flushable {
	type CanvasType = GenericCanvas<Self>;
	
	fn into_canvas(self) -> Self::CanvasType where Self: Sized {
		return GenericCanvas::new(self);
	}
}

#[derive(Clone, PartialEq, Debug)]
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

		for button in self.iter() {
			if button.get(self) != button.get_old(self) {
				let color = button.get(self);
				changes.push((button.x(), button.y(), color));
			}
		}

		if changes.len() > 0 {
			self.backend.flush(&changes)?;
		}

		self.curr_state = self.new_state.clone();

		return Ok(());
	}
}

// Now, the canvas layout stuff......

use std::collections::HashMap;

struct LayoutDevice {
	canvas: Box<dyn Canvas>,
	x: u32,
	y: u32,
}

#[allow(clippy::borrowed_box)] // YES I KNOW WHAT I'M DOING (I don't even know if I'm being ironic)
fn get_boxed_type<T: ?Sized + std::any::Any>(_value: &Box<T>) -> &'static str {
	return std::any::type_name::<T>();
}

impl std::fmt::Debug for LayoutDevice {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		return f.write_fmt(
			format_args!(
				"{} from ({}|{}) to incl ({}|{})",
				get_boxed_type(&self.canvas),
				self.x,
				self.y,
				self.x + self.canvas.bounding_box_width() - 1,
				self.y + self.canvas.bounding_box_height() - 1,
			)
		);
	}
}

#[derive(Default, Debug)]
pub struct CanvasLayout {
	devices: Vec<LayoutDevice>,
	// Maps coordinates to a specific LayoutDevice, specified by an index into the vector
	coordinate_map: HashMap<(u32, u32), usize>,
}

impl CanvasLayout {
	pub fn new() -> Self {
		return Self {
			devices: vec![],
			coordinate_map: HashMap::new(),
		};
	}

	pub fn add(&mut self, canvas: impl Canvas + 'static, x: u32, y: u32) {
		let index = self.devices.len(); // The index of soon-to-be inserted object

		for button in canvas.iter() {
			self.coordinate_map.insert((x + button.x(), y + button.y()), index);
		}

		let canvas_box = Box::new(canvas) as Box<dyn Canvas>; // Static -> dynamic dispatch

		// TODO: check collision!

		self.devices.push(LayoutDevice { canvas: canvas_box, x, y });
	}
}

impl Canvas for CanvasLayout {
	fn bounding_box_width(&self) -> u32 {
		return self.devices.iter()
				.map(|device| device.x + device.canvas.bounding_box_width())
				.max().unwrap_or(0);
	}
	
	fn bounding_box_height(&self) -> u32 {
		return self.devices.iter()
				.map(|device| device.y + device.canvas.bounding_box_height())
				.max().unwrap_or(0);
	}
	
	fn is_valid(&self, x: u32, y: u32) -> bool {
		return self.coordinate_map.contains_key(&(x, y));
	}
	
	fn get_unchecked(&self, x: u32, y: u32) -> Color {
		let device = &self.devices[*self.coordinate_map.get(&(x, y)).unwrap()];
		return device.canvas.get_unchecked(x - device.x, y - device.y);
	}
	
	fn set_unchecked(&mut self, x: u32, y: u32, color: Color) {
		let device = &mut self.devices[*self.coordinate_map.get(&(x, y)).unwrap()];
		return device.canvas.set_unchecked(x - device.x, y - device.y, color);
	}
	
	fn get_old_unchecked(&self, x: u32, y: u32) -> Color {
		let device = &self.devices[*self.coordinate_map.get(&(x, y)).unwrap()];
		return device.canvas.get_old_unchecked(x - device.x, y - device.y);
	}
	
	fn flush(&mut self) -> anyhow::Result<()> {
		for device in &mut self.devices {
			device.canvas.flush()?;
		}
		return Ok(());
	}
}