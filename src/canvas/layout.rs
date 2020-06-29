use super::*;
use crate::Color;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Rotation {
	None,
	Left,
	Right,
	UpsideDown
}

impl Default for Rotation {
	fn default() -> Self { Self::None }
}

impl std::ops::Neg for Rotation {
	type Output = Self;

	fn neg(self) -> Self {
		match self {
			Self::None => Self::None,
			Self::UpsideDown => Self::UpsideDown,
			Self::Left => Self::Right,
			Self::Right => Self::Left
		}
	}
}

impl Rotation {
	pub fn translate(self, x: u32, y: u32, width: u32, height: u32) -> (u32, u32) {
		match self {
			Self::None => (x, y),
			Self::UpsideDown => (width - x - 1, height - y - 1),
			Self::Left => (y, width - x - 1),
			Self::Right => (height - y - 1, x),
		}
	}
}

struct LayoutDevice<'a> {
	canvas: Box<dyn Canvas + 'a>,
	rotation: Rotation,
	x: u32,
	y: u32,
}

unsafe impl Sync for LayoutDevice<'_> {} // fuck it

impl LayoutDevice<'_> {
	fn to_local(&self, x: u32, y: u32) -> (u32, u32) {
		(-self.rotation).translate(x - self.x, y - self.y,
				self.canvas.bounding_box_width(), self.canvas.bounding_box_height())
	}

	fn to_global(&self, x: u32, y: u32) -> (u32, u32) {
		let (rotated_x, rotated_y) = self.rotation.translate(x, y,
				self.canvas.bounding_box_width(), self.canvas.bounding_box_height());
		(rotated_x + self.x, rotated_y + self.y)
	}
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
			devices: Vec::with_capacity(10), // HACKJOB HACKJOB HACKJOB I NEED TO PREVENT REALLOCATIONS SO THAT THE CALLBACK WRAPPER DOESNT READ FROM UNINITIALIZED MEM so 10 ought to be enough hopefully
			coordinate_map: HashMap::new(),
			callback: std::sync::Arc::new(Box::new(callback)),
		};
	}

	/// Create a new CanvasLayout, plus an input handler object that you can use to poll messages.
	pub fn new_polling() -> (Self, CanvasLayoutPoller) {
		let (sender, receiver) = std::sync::mpsc::sync_channel(50);
		let canvas = Self::new(move |msg| sender.send(msg)
				.expect("Message receiver has hung up (this shouldn't happen)"));
		
		let poller = CanvasLayoutPoller { receiver };

		(canvas, poller)
	}

	/// Add a new device to this canvas layout, at the specified `x` and `y` coordinate.
	/// 
	/// The usage of this method is a bit awkward out of necessity. You need to provide a closure
	/// which, when called with a message callback, is expected to return a `Canvas` that is set up
	/// to deliver messsages to the provided message callback.
	/// 
	/// Any `Error`s from the closure will be propagated.
	/// 
	/// Example:
	/// ```rust
	/// canvas_layout.add(0, 0, |callback| launchy::mk2::Canvas::guess(callback))?;
	/// 
	/// // or even nested layouts:
	/// canvas_layout.add(0, 0, |callback| {
	/// 	let mut inner_canvas_layout = CanvasLayout::new(callback);
	/// 	inner_canvas_layout.add(0, 0, |inner_callback| launchy::mk2::Canvas::guess(inner_callback))
	/// })?;
	/// ```
	/// 
	/// If you want an easier way to add devices, see `add_by_guess`.
	pub fn add<C: 'a + Canvas, F, E>(&mut self,
		x_offset: u32,
		y_offset: u32,
		rotation: Rotation,
		creator: F
	) -> Result<(), E>
		where F: FnOnce(Box<dyn Fn(CanvasMessage) + Send + 'a>) -> Result<C, E> {
		
		let callback = self.callback.clone();
		let layout_device_container: std::sync::Arc<std::sync::Mutex<Option<&LayoutDevice>>> = std::sync::Arc::new(std::sync::Mutex::new(None));
		let layout_device_container_inner = layout_device_container.clone();
		let canvas = (creator)(Box::new(move |msg| {
			let layout_device = if let Some(a) = *layout_device_container_inner.lock().unwrap() { a } else { return };

			let (x, y) = layout_device.to_global(msg.x(), msg.y());
			match msg {
				CanvasMessage::Press { .. } => (callback)(CanvasMessage::Press { x, y }),
				CanvasMessage::Release { .. } => (callback)(CanvasMessage::Release { x, y }),
			}
		}))?;
		
		let index = self.devices.len(); // The index of soon-to-be inserted object
		let layout_device = LayoutDevice {
			canvas: Box::new(canvas),
			rotation, x: x_offset, y: y_offset
		};
		
		for btn in layout_device.canvas.iter() {
			let translated_coords = layout_device.to_global(btn.x(), btn.y());
			let old_value = self.coordinate_map.insert(translated_coords, index);
			
			// check for overlap
			if let Some(old_index) = old_value {
				panic!("Canvas is overlapping with canvas {} (zero-indexed) at ({}|{})!",
				old_index, translated_coords.0, translated_coords.1);
			}
		}
		
		self.devices.push(layout_device);

		// TODO: this Arc<Mutex> thing is a very hacky solution
		*layout_device_container.lock().unwrap() = Some(unsafe {
			&*(&self.devices[self.devices.len() - 1] as *const LayoutDevice)
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
	pub fn add_by_guess<E: 'a + DeviceCanvasTrait>(&mut self,
		x: u32, y: u32,
	) -> anyhow::Result<()> {

		self.add(x, y, Rotation::None, DeviceCanvas::<E::Spec>::guess)
	}

	/// Like `add_by_guess`, but with a parameter for the rotation of the Launchpad.
	pub fn add_by_guess_rotated<E: 'a + DeviceCanvasTrait>(&mut self,
		x: u32, y: u32, rotation: Rotation,
	) -> anyhow::Result<()> {

		self.add(x, y, rotation, DeviceCanvas::<E::Spec>::guess)
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
		let (x, y) = device.to_local(x, y);
		device.canvas.get_unchecked(x, y)
	}
	
	fn set_unchecked(&mut self, x: u32, y: u32, color: Color) {
		let device = &mut self.devices[*self.coordinate_map.get(&(x, y)).unwrap()];
		let (x, y) = device.to_local(x, y);
		device.canvas.set_unchecked(x, y, color)
	}
	
	fn get_old_unchecked(&self, x: u32, y: u32) -> Color {
		let device = &self.devices[*self.coordinate_map.get(&(x, y)).unwrap()];
		let (x, y) = device.to_local(x, y);
		device.canvas.get_old_unchecked(x, y)
	}
	
	fn flush(&mut self) -> anyhow::Result<()> {
		for device in &mut self.devices {
			device.canvas.flush()?;
		}
		return Ok(());
	}
}