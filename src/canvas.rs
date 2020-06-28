use crate::Color;


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

pub trait DeviceSpec {
	const BOUNDING_BOX_WIDTH: u32;
	const BOUNDING_BOX_HEIGHT: u32;

	type Input: crate::InputDevice;
	type Output: crate::OutputDevice;

	fn is_valid(x: u32, y: u32) -> bool;
	fn flush(output: &mut Self::Output, changes: &[(u32, u32, crate::Color)]) -> anyhow::Result<()>;
	fn convert_message(msg: <Self::Input as crate::InputDevice>::Message) -> Option<CanvasMessage>;
}

pub struct DeviceCanvas<'a, Spec: DeviceSpec> {
	_input: crate::InputDeviceHandler<'a>,
	output: Spec::Output,
	curr_state: crate::util::Array2d<crate::Color>,
	new_state: crate::util::Array2d<crate::Color>,
}

impl<'a, Spec: DeviceSpec> DeviceCanvas<'a, Spec> {
	pub fn guess(mut callback: impl FnMut(CanvasMessage) + Send + 'a) -> anyhow::Result<Self> {
		use crate::midi_io::{InputDevice, OutputDevice};

		let _input = Spec::Input::guess(move |msg| {
			if let Some(msg) = Spec::convert_message(msg) {
				(callback)(msg);
			}
		})?;
		let output = Spec::Output::guess()?;
		
		let curr_state = crate::util::Array2d::new(
			Spec::BOUNDING_BOX_WIDTH as usize,
			Spec::BOUNDING_BOX_HEIGHT as usize,
		);
		let new_state = crate::util::Array2d::new(
			Spec::BOUNDING_BOX_WIDTH as usize,
			Spec::BOUNDING_BOX_HEIGHT as usize,
		);

		Ok(Self { _input, output, curr_state, new_state })
	}
}

impl<Spec: DeviceSpec> crate::Canvas for DeviceCanvas<'_, Spec> {
	fn bounding_box_width(&self) -> u32 { Spec::BOUNDING_BOX_WIDTH }
	fn bounding_box_height(&self) -> u32 { Spec::BOUNDING_BOX_HEIGHT }
	fn is_valid(&self, x: u32, y: u32) -> bool { Spec::is_valid(x, y) }

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
			Spec::flush(&mut self.output, &changes)?;
		}

		self.curr_state = self.new_state.clone();

		return Ok(());
	}
}

// Now, the canvas layout stuff......

use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum CanvasMessage {
	Press { x: u32, y: u32 },
	Release { x: u32, y: u32 },
}

struct LayoutDevice<'a> {
	canvas: Box<dyn Canvas + 'a>,
	x: u32,
	y: u32,
}

pub struct CanvasLayout<'a> {
	devices: Vec<LayoutDevice<'a>>,
	// Maps coordinates to a specific LayoutDevice, specified by an index into the vector
	coordinate_map: HashMap<(u32, u32), usize>,
	callback: Arc<Box<dyn Fn(CanvasMessage) + Send + Sync + 'a>>,
}

pub struct CanvasLayoutPoller {
	receiver: std::sync::mpsc::Receiver<CanvasMessage>,
}

impl crate::MsgPollingWrapper for CanvasLayoutPoller {
	type Message = CanvasMessage;

	fn receiver(&self) -> &std::sync::mpsc::Receiver<Self::Message> { &self.receiver }
}

use std::sync::Arc;

impl<'a> CanvasLayout<'a> {
	pub fn new(callback: impl Fn(CanvasMessage) + Send + Sync + 'a) -> Self {
		return Self {
			devices: vec![],
			coordinate_map: HashMap::new(),
			callback: Arc::new(Box::new(callback)),
		};
	}

	pub fn new_polling() -> (CanvasLayoutPoller, Self) {
		let (sender, receiver) = std::sync::mpsc::sync_channel(50);
		return (CanvasLayoutPoller { receiver }, Self::new(move |msg| sender.send(msg)
				.expect("Message receiver has hung up (this shouldn't happen)")));
	}

	pub fn add<C: 'a + Canvas, F, E>(&mut self, x: u32, y: u32, creator: F) -> Result<(), E>
			where F: FnOnce(Box<dyn Fn(CanvasMessage) + Send + 'a>) -> Result<C, E> {
		
		let callback = self.callback.clone();
		let canvas = (creator)(Box::new(move |msg| {
			match msg {
				CanvasMessage::Press { x: msg_x, y: msg_y } => {
					(callback)(CanvasMessage::Press { x: msg_x + x, y: msg_y + y });
				},
				CanvasMessage::Release { x: msg_x, y: msg_y } => {
					(callback)(CanvasMessage::Release { x: msg_x + x, y: msg_y + y });
				},
			}
		}))?;

		self.devices.push(LayoutDevice {
			canvas: Box::new(canvas),
			x, y
		});

		return Ok(());
	}

	pub fn add_by_guess<S: 'a + DeviceSpec>(&mut self, x: u32, y: u32) -> anyhow::Result<()> {
		self.add(x, y, DeviceCanvas::<S>::guess)
	}
}

impl Canvas for CanvasLayout<'_> {
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