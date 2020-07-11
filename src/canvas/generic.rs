use super::*;


/// Launchpad's implement this trait to signify how they can be used as a [`Canvas`]. Based on this
/// specification, [`DeviceCanvas`] provides a generic [`Canvas`] implemention that can be used for all
/// devices.
/// 
/// You as a user of this library will not need to use this trait directly.
pub trait DeviceSpec {
	/// The width of the smallest rectangle that still fully encapsulates the shape of this device
	const BOUNDING_BOX_WIDTH: u32;
	/// The height of the smallest rectangle that still fully encapsulates the shape of this device
	const BOUNDING_BOX_HEIGHT: u32;
	/// How many different colors can be shown per channel. As an example; the MK2 uses 6 bit color,
	/// so it supports color values from 0 up to 63 - in total 64 values.
	const COLOR_PRECISION: u16;

	/// The input handler type
	type Input: crate::InputDevice;
	/// The output handler type
	type Output: crate::OutputDevice;

	/// Returns whether the point at the given `x` and `y` coordinates are in bounds
	fn is_valid(x: u32, y: u32) -> bool;
	
	/// Flush the changes, as specified by `changes`, to the given underlying output handler.
	/// 
	/// `changes` is a slice of tuples `(u32, u32, (u8, u8, u8))`, where the first element is the x
	/// coordinate, the second element is the y coordinate, and the third element is an RGB color
	/// tuple, according to `COLOR_PRECISION`.
	fn flush(
		canvas: &mut crate::DeviceCanvas<Self>,
		changes: &[(u32, u32, (u8, u8, u8))])
	-> Result<(), crate::MidiError>
		where Self: Sized;

	/// Convert a message from the underlying input handler into an abstract CanvasMessage. If the
	/// low-level message has no CanvasMessage equivalent, i.e. if it's irrelevant in a canvas
	/// context, None is returned.
	fn convert_message(msg: <Self::Input as crate::InputDevice>::Message) -> Option<CanvasMessage>;

	/// Optional code to setup this device for canvas usage
	fn setup(output: &mut Self::Output) -> Result<(), crate::MidiError> {
		let _ = output;
		Ok(())
	}
}

/// Utility to be able to process messages from a CanvasLayout by polling
pub struct DeviceCanvasPoller {
	receiver: std::sync::mpsc::Receiver<CanvasMessage>,
}

impl crate::MsgPollingWrapper for DeviceCanvasPoller {
	type Message = CanvasMessage;

	fn receiver(&self) -> &std::sync::mpsc::Receiver<Self::Message> { &self.receiver }
}

/// A generic [`Canvas`] implementation for all launchpads, that relies on a [`DeviceSpec`]. You as a
/// user of the library don't need to access this struct directly. Use the "Canvas" type aliases
/// that each launchpad module provides, for example `launchy::mk2::Canvas` or
/// `launchy::s::Canvas`.
pub struct DeviceCanvas<'a, Spec: DeviceSpec> {
	_input: crate::InputDeviceHandler<'a>,
	pub(crate) output: Spec::Output,
	curr_state: crate::util::Array2d<crate::Color>,
	new_state: crate::util::Array2d<crate::Color>,
	// This is a debug variable to be able to see how many messages I'm actually spewing out.
	num_sent_changes: usize,
}

impl<'a, Spec: DeviceSpec> DeviceCanvas<'a, Spec> {
	/// Create a new canvas by guessing both input and output MIDI connection by their name. If you
	/// need precise control over the specific MIDI connections that will be used, use
	/// [`DeviceCanvas::from_ports`] instead // TODO: not implemented yet
	pub fn guess(
		mut callback: impl FnMut(CanvasMessage) + Send + 'a
	) -> Result<Self, crate::MidiError> {
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

		Ok(Self { _input, output, curr_state, new_state, num_sent_changes: 0 })
	}

	pub fn guess_polling() -> Result<(Self, DeviceCanvasPoller), crate::MidiError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let canvas = Self::guess(move |msg| {
			sender.send(msg)
				.expect("Message receiver has hung up (this shouldn't happen)")
		})?;
		
		let poller = DeviceCanvasPoller { receiver };

		Ok((canvas, poller))
	}
}

#[doc(hidden)] // this is crap workaround and shouldn't be seen by user directly
pub trait DeviceCanvasTrait {
	type Spec: DeviceSpec;
}

impl<S: DeviceSpec> DeviceCanvasTrait for DeviceCanvas<'_, S> {
	type Spec = S;
}

impl_traits_for_canvas!(<'a, S: DeviceSpec>, DeviceCanvas);

impl<Spec: DeviceSpec> crate::Canvas for DeviceCanvas<'_, Spec> {
	fn bounding_box_width(&self) -> u32 { Spec::BOUNDING_BOX_WIDTH }
	fn bounding_box_height(&self) -> u32 { Spec::BOUNDING_BOX_HEIGHT }
	fn is_valid(&self, x: u32, y: u32) -> bool { Spec::is_valid(x, y) }
	fn lowest_visible_brightness(&self) -> f32 { 1.0 / Spec::COLOR_PRECISION as f32 }

	fn get_old_unchecked_ref(&self, x: u32, y: u32) -> &Color {
		self.curr_state.get_ref(x as usize, y as usize)
	}

	fn get_new_unchecked_mut(&mut self, x: u32, y: u32) -> &mut Color {
		self.new_state.get_mut(x as usize, y as usize)
	}

	fn get_new_unchecked_ref(&self, x: u32, y: u32) -> &Color {
		self.new_state.get_ref(x as usize, y as usize)
	}

	fn flush(&mut self) -> Result<(), crate::MidiError> {
		let mut changes: Vec<(u32, u32, (u8, u8, u8))> = Vec::with_capacity(9 * 9);

		for pad in self.iter() {
			let old = self[pad].quantize(Spec::COLOR_PRECISION);
			let new = self.at_new(pad).quantize(Spec::COLOR_PRECISION);
			if new != old {
				changes.push((pad.x as u32, pad.y as u32, new));
			}
		}

		if !changes.is_empty() {
			use crate::midi_io::OutputDevice;
			self.num_sent_changes += changes.len();
			if self.num_sent_changes / 1000 != (self.num_sent_changes - changes.len()) / 1000 {
				println!("{}: we're at {} total transmitted changes now",
						Spec::Output::MIDI_DEVICE_KEYWORD,
						self.num_sent_changes,
				);
			}

			Spec::flush(self, &changes)?;
		}

		self.curr_state = self.new_state.clone();

		return Ok(());
	}
}