//!	The canvas module provides incredibly useful abstractions over the low-level devices.
//! 
//! It allows you to write code that works on any Launchpad - be it the original Launchpad from
//! 2009, the Launchpad MK2, or even the 12 buttons on a Launch Control!
//! 
//! Additionally, you can chain multiple devices together as if they were a single device, using
//! `CanvasLayout`.
//! 
//! **Please look into the documentation of `Canvas`, `CanvasIterator` and `CanvasLayout` for
//! detailed documentation and examples!**

use crate::Color;

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

/// Launchpad's implement this trait to signify how they can be used as a `Canvas`. Based on this
/// specification, `DeviceCanvas` provides a generic `Canvas` implemention that can be used for all
/// devices.
/// 
/// You as a user of this library will not need to use this trait directly.
pub trait DeviceSpec {
	/// The width of the smallest rectangle that still fully encapsulates the shape of this device
	const BOUNDING_BOX_WIDTH: u32;
	/// The height of the smallest rectangle that still fully encapsulates the shape of this device
	const BOUNDING_BOX_HEIGHT: u32;

	/// The input handler type
	type Input: crate::InputDevice;
	/// The output handler type
	type Output: crate::OutputDevice;

	/// Returns whether the point at the given `x` and `y` coordinates are in bounds
	fn is_valid(x: u32, y: u32) -> bool;
	/// Flush the changes, as specified by `changes`, to the given underlying output handler.
	/// 
	/// `changes` is a slice of tuples `(u32, u32, Color)`, where the first element in the x
	/// coordinate, the second element is the y coordinate, and the third element is the new color
	/// at that point.
	fn flush(output: &mut Self::Output, changes: &[(u32, u32, crate::Color)]) -> anyhow::Result<()>;
	/// Convert a message from the underlying input handler into an abstract CanvasMessage. If the
	/// low-level message has no CanvasMessage equivalent, i.e. if it's irrelevant in a canvas
	/// context, None is returned.
	fn convert_message(msg: <Self::Input as crate::InputDevice>::Message) -> Option<CanvasMessage>;

	/// Optional code to setup this device for canvas usage
	fn setup(output: &mut Self::Output) -> anyhow::Result<()> {
		let _ = output;
		Ok(())
	}
}

/// A generic `Canvas` implementation for all launchpads, that relies on a `DeviceSpec`. You as a
/// user of the library don't need to access this struct directly. Use the "Canvas" type aliases
/// that each launchpad module provides, for example `launchy::mk2::Canvas` or
/// `launchy::s::Canvas`.
pub struct DeviceCanvas<'a, Spec: DeviceSpec> {
	_input: crate::InputDeviceHandler<'a>,
	output: Spec::Output,
	curr_state: crate::util::Array2d<crate::Color>,
	new_state: crate::util::Array2d<crate::Color>,
}

impl<'a, Spec: DeviceSpec> DeviceCanvas<'a, Spec> {
	/// Create a new canvas by guessing both input and output MIDI connection by their name. If you
	/// need precise control over the specific MIDI connections that will be used, use
	/// `DeviceCanvas::from_ports()` instead // TODO: not implemented yet
	pub fn guess(mut callback: impl FnMut(CanvasMessage) + Send + 'a) -> anyhow::Result<Self> {
		use crate::midi_io::{InputDevice, OutputDevice};

		let _input = Spec::Input::guess(move |msg| {
			if let Some(msg) = Spec::convert_message(msg) {
				(callback)(msg);
			}
		})?;
		let mut output = Spec::Output::guess()?;
		Spec::setup(&mut output)?;
		
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

#[doc(hidden)] // this is crap workaround and won't be needed by user directly
pub trait DeviceCanvasTrait {
	type Spec: DeviceSpec;
}

impl<S: DeviceSpec> DeviceCanvasTrait for DeviceCanvas<'_, S> {
	type Spec = S;
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

		if !changes.is_empty() {
			Spec::flush(&mut self.output, &changes)?;
		}

		self.curr_state = self.new_state.clone();

		return Ok(());
	}
}

// Now, the canvas layout stuff......

use std::collections::HashMap;

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

/// Imagine this - you have multiple launchpads, you line them up, and now you use the Launchpads
/// as if they were a single device?! You can do that, with `CanvasLayout`.
/// 
/// Create a layout, add `Canvas`es to it at the position where they appear on your table, and
/// you're ready to rock!
/// 
/// Example:
/// ```rust
/// let mut canvas_layout = CanvasLayout::new(|msg| println!("Got a message: {:?}", msg));
/// 
/// // Assuming you have a Launchpad MK2 and a Launchpad S lying next to it:
/// canvas_layout.add_by_guess::<launchy::mk2::Canvas>(0, 0);
/// canvas_layout.add_by_guess::<launchy::s::Canvas>(9, 0);
/// 
/// // Light the entire canvas layout red - i.e. both Launchpads will be red
/// for button in canvas_layout.iter() {
/// 	button.set(&mut canvas_layout, launchy::Color::RED);
/// }
/// ```
pub struct CanvasLayout<'a> {
	devices: Vec<LayoutDevice<'a>>,
	// Maps coordinates to a specific LayoutDevice, specified by an index into the vector
	coordinate_map: HashMap<(u32, u32), usize>,
	callback: std::sync::Arc<Box<dyn Fn(CanvasMessage) + Send + Sync + 'a>>,
}

/// Utility to be able to process messages from a CanvasLayout by polling
pub struct CanvasLayoutPoller {
	receiver: std::sync::mpsc::Receiver<CanvasMessage>,
}

impl crate::MsgPollingWrapper for CanvasLayoutPoller {
	type Message = CanvasMessage;

	fn receiver(&self) -> &std::sync::mpsc::Receiver<Self::Message> { &self.receiver }
}

impl<'a> CanvasLayout<'a> {
	/// Create a new CanvasLayout that sends messages to the provided callback. The callback must
	/// implement `Fn` because it may be called from multiple devices concurrently.
	pub fn new(callback: impl Fn(CanvasMessage) + Send + Sync + 'a) -> Self {
		return Self {
			devices: vec![],
			coordinate_map: HashMap::new(),
			callback: std::sync::Arc::new(Box::new(callback)),
		};
	}

	/// Create a new CanvasLayout, plus an input handler object that you can use to poll messages.
	pub fn new_polling() -> (CanvasLayoutPoller, Self) {
		let (sender, receiver) = std::sync::mpsc::sync_channel(50);
		return (CanvasLayoutPoller { receiver }, Self::new(move |msg| sender.send(msg)
				.expect("Message receiver has hung up (this shouldn't happen)")));
	}

	/// Add a new device to this canvas layout, at the specified `x` and `y` coordinate.
	/// 
	/// The usage of this method is a bit awkward out of necessity. You need to provide a closure
	/// which will be called with a message callback. The closure is expected to return a `Canvas`
	/// that is set up to deliver messsages to the passed message callback.
	/// 
	/// Example:
	/// ```rust
	/// canvas_layout.add(0, 0, |callback| launchy::mk2::Canvas::guess(callback));
	/// 
	/// // or even nested layouts:
	/// canvas_layout.add(0, 0, |callback| {
	/// 	let mut inner_canvas_layout = CanvasLayout::new(callback);
	/// 	inner_canvas_layout.add(0, 0, |inner_callback| launchy::mk2::Canvas::guess(inner_callback));
	/// });
	/// ```
	/// 
	/// If you want an easier way to add devices, see `add_by_guess`.
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

		let index = self.devices.len(); // The index of soon-to-be inserted object

		for button in canvas.iter() {
			let translated_coords = (x + button.x(), y + button.y());
			let old_value = self.coordinate_map.insert(translated_coords, index);

			// check for overlap
			if let Some(old_index) = old_value {
				panic!("Canvas is overlapping with canvas {} (zero-indexed) at ({}|{})!",
						old_index, translated_coords.0, translated_coords.1);
			}
		}

		self.devices.push(LayoutDevice {
			canvas: Box::new(canvas),
			x, y
		});

		return Ok(());
	}

	/// Add a new device to this canvas, at the specified `x` and `y` coordinates. The MIDI
	/// connections used for communication with the underlying hardware are determined by guessing
	/// based on the device name.
	/// 
	/// Specifiy the type of device using a generic Canvas type parameter.
	/// 
	/// Example
	/// ```rust
	/// // Assuming a Launchpad MK2 and a Launchpad S next to it:
	/// canvas_layout.add_by_guess::<launchy::mk2::Canvas>(0, 0);
	/// canvas_layout.add_by_guess::<launchy::s::Canvas>(9, 0);
	/// ```
	pub fn add_by_guess<E: 'a + DeviceCanvasTrait>(&mut self, x: u32, y: u32) -> anyhow::Result<()> {
		self.add(x, y, DeviceCanvas::<E::Spec>::guess)
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