use crate::ok_or_continue;
use anyhow::{Context, anyhow};
use midir::{MidiOutput, MidiOutputConnection, MidiInput, MidiInputConnection, MidiInputPort};


fn guess_port<T: midir::MidiIO>(midi_io: &T, keyword: &str) -> Option<T::Port> {
	for port in midi_io.ports() {
		let name = ok_or_continue!(midi_io.port_name(&port));
		
		if name.contains(keyword) {
			return Some(port);
		}
	}

	return None;
}

pub trait OutputDevice where Self: Sized {
	const MIDI_CONNECTION_NAME: &'static str;
	const MIDI_DEVICE_KEYWORD: &'static str;

	/// Initiate from an existing midir connection.
	fn from_connection(connection: MidiOutputConnection) -> anyhow::Result<Self>;

	fn send(&mut self, bytes: &[u8]) -> anyhow::Result<()>;

	fn guess() -> anyhow::Result<Self> {
		let midi_output = MidiOutput::new(crate::APPLICATION_NAME)
				.context("Couldn't create MidiOutput object")?;

		let port = guess_port(&midi_output, Self::MIDI_DEVICE_KEYWORD)
				.context(format!("No '{}' output device found", Self::MIDI_DEVICE_KEYWORD))?;
		
		let connection = midi_output
				.connect(&port, Self::MIDI_CONNECTION_NAME)
				.map_err(|_| anyhow!("Failed to connect to port"))?;
		
		return Self::from_connection(connection);
	}
}

pub struct InputDeviceHandler<'a> {
	#[allow(dead_code)]
	connection: MidiInputConnection<'a, ()>
}

pub struct InputDeviceHandlerPolling<'a, Message> {
	#[allow(dead_code)]
	connection: MidiInputConnection<'a, ()>,
	receiver: std::sync::mpsc::Receiver<Message>,
}

impl<Message> InputDeviceHandlerPolling<'_, Message> {
	/// Wait for a message to arrive, and return that. For a non-block variant, see `try_recv()`.
	pub fn recv(&self) -> Message {
		return self.receiver.recv()
				.expect("Message sender has hung up - please report a bug");
	}

	/// If there is a pending message, return that. Otherwise, return `None`.
	/// 
	/// This function does not block.
	pub fn try_recv(&self) -> Option<Message> {
		use std::sync::mpsc::TryRecvError;
		match self.receiver.try_recv() {
			Ok(msg) => return Some(msg),
			Err(TryRecvError::Empty) => return None,
			Err(TryRecvError::Disconnected) => panic!("Message sender has hung up - please report a bug"),
		}
	}

	/// Receives a single message. If no message arrives within the timespan specified by `timeout`,
	/// `None` is returned.
	pub fn recv_timeout(&self, timeout: std::time::Duration) -> Option<Message> {
		use std::sync::mpsc::RecvTimeoutError;
		match self.receiver.recv_timeout(timeout) {
			Ok(msg) => return Some(msg),
			Err(RecvTimeoutError::Timeout) => return None,
			Err(RecvTimeoutError::Disconnected) => panic!("Message sender has hung up - please report a bug"),
		}
	}

	/// Returns an iterator over all arriving messages. The iterator will only return when the
	/// MIDI connection has been dropped.
	/// 
	/// For an iteration method that doesn't block, but returns immediately when there are no more
	/// pending messages, see `iter_pending`.
	pub fn iter(&self) -> impl Iterator<Item=Message> + '_ {
		return self.receiver.iter();
	}

	/// Returns an iterator over the currently pending messages. As soon as all pending messages
	/// have been iterated over, the iterator will return.
	/// 
	/// For an iteration method that will block, waiting for new messages to arrive, see `iter()`.
	pub fn iter_pending(&self) -> impl Iterator<Item=Message> + '_ {
		return self.receiver.try_iter();
	}

	/// Drain of any pending messages. This is useful on Launchpad startup - the Launchpad has the
	/// weird property that any button inputs while disconnected queue up and will all be released
	/// at the same time as soon as someone connects to it. In most cases you don't want to deal
	/// with those stale messages though - in those cases, call `drain()` after establishing the
	/// connection.
	/// 
	/// This function returns the number of messages that were discarded.
	pub fn drain(&self) -> usize {
		return self.iter_pending().count();
	}
}

pub trait InputDevice {
	const MIDI_CONNECTION_NAME: &'static str;
	const MIDI_DEVICE_KEYWORD: &'static str;
	type Message;

	fn decode_message(timestamp: u64, data: &[u8]) -> Self::Message;

	#[must_use = "If not saved, the connection will be immediately dropped"]
	fn from_port<'a, F>(midi_input: MidiInput, port: &MidiInputPort, mut user_callback: F)
			-> anyhow::Result<InputDeviceHandler<'a>>
			where F: FnMut(Self::Message) + Send + 'a {
		
		let midir_callback = move |timestamp: u64, data: &[u8], _: &mut _| {
			let msg = Self::decode_message(timestamp, data);
			(user_callback)(msg);
		};
		
		let connection = midi_input.connect(port, Self::MIDI_CONNECTION_NAME, midir_callback, ())
				.map_err(|_| anyhow!("Failed to connect to port"))?;
		
		return Ok(InputDeviceHandler { connection });
	}

	#[must_use = "If not saved, the connection will be immediately dropped"]
	fn from_port_polling(midi_input: MidiInput, port: &MidiInputPort)
			-> anyhow::Result<InputDeviceHandlerPolling<'static, Self::Message>>
			where Self::Message: Send + 'static {
		
		let (sender, receiver) = std::sync::mpsc::channel();
		let midir_callback = move |timestamp: u64, data: &[u8], _: &mut _| {
			let msg = Self::decode_message(timestamp, data);
			// The following statement can only panic when the receiver was dropped but the
			// connection is still alive. This can't happen by accident I think, because the
			// user would have to destructure the input device handler in order to get the
			// connection and the receiver seperately, in order to drop one but not the other -
			// but if he does that it's his fault that he gets a panic /shrug
			sender.send(msg).expect("Message receiver has hung up (this shouldn't happen)");
		};
		
		let connection = midi_input.connect(port, Self::MIDI_CONNECTION_NAME, midir_callback, ())
				.map_err(|_| anyhow!("Failed to connect to port"))?;
		
		return Ok(InputDeviceHandlerPolling { connection, receiver });
	}
	
	/// Search the midi devices and choose the first midi device matching the wanted Launchpad type.
	#[must_use = "If not saved, the connection will be immediately dropped"]
	fn guess<'a, F>(user_callback: F) -> anyhow::Result<InputDeviceHandler<'a>>
			where F: FnMut(Self::Message) + Send + 'a {
		
		let midi_input = MidiInput::new(crate::APPLICATION_NAME)
				.context("Couldn't create MidiInput object")?;

		let port = guess_port(&midi_input, Self::MIDI_DEVICE_KEYWORD)
				.context(format!("No '{}' input device found", Self::MIDI_DEVICE_KEYWORD))?;
		
		return Self::from_port(midi_input, &port, user_callback);
	}

	/// Search the midi devices and choose the first midi device matching the wanted Launchpad type.
	#[must_use = "If not saved, the connection will be immediately dropped"]
	fn guess_polling<'a>() -> anyhow::Result<InputDeviceHandlerPolling<'a, Self::Message>>
			where Self::Message: Send + 'static {
		
		let midi_input = MidiInput::new(crate::APPLICATION_NAME)
				.context("Couldn't create MidiInput object")?;

		let port = guess_port(&midi_input, Self::MIDI_DEVICE_KEYWORD)
				.context(format!("No '{}' input device found", Self::MIDI_DEVICE_KEYWORD))?;
		
		return Self::from_port_polling(midi_input, &port);
	}
}