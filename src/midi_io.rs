use log::debug;
use midir::{MidiInput, MidiInputConnection, MidiInputPort, MidiOutput, MidiOutputConnection};

fn guess_port<T: midir::MidiIO>(midi_io: &T, keyword: &str) -> Option<T::Port> {
    for port in midi_io.ports() {
        let name = match midi_io.port_name(&port) {
            Ok(name) => name,
            Err(_) => continue,
        };
        debug!("Considering MIDI port: '{}'", name);
        if name.contains(keyword) {
            debug!("Found matching MIDI port: '{}'", name);
            return Some(port);
        }
    }
    debug!("No MIDI port found with keyword: '{}'", keyword);
    None
}

pub trait OutputDevice
where
    Self: Sized,
{
    const MIDI_CONNECTION_NAME: &'static str;
    const MIDI_DEVICE_KEYWORD: &'static str;

    /// Initiate from an existing midir connection.
    fn from_connection(connection: MidiOutputConnection) -> Result<Self, crate::MidiError>;

    fn send(&mut self, bytes: &[u8]) -> Result<(), crate::MidiError>;

    fn guess() -> Result<Self, crate::MidiError> {
        debug!(
            "Attempting to guess output device with keyword: '{}'",
            Self::MIDI_DEVICE_KEYWORD
        );
        let midi_output = MidiOutput::new(crate::APPLICATION_NAME)?;
        let port = guess_port(&midi_output, Self::MIDI_DEVICE_KEYWORD).ok_or(
            crate::MidiError::NoPortFound {
                keyword: Self::MIDI_DEVICE_KEYWORD,
            },
        )?;
        let connection = midi_output.connect(&port, Self::MIDI_CONNECTION_NAME)?;
        debug!(
            "Successfully connected to output device: '{}'",
            Self::MIDI_DEVICE_KEYWORD
        );
        Self::from_connection(connection)
    }
}

/// A handler for a Launchpad input connection. This variant is used when an input connection is
/// initiated with callback
pub struct InputDeviceHandler {
    _connection: MidiInputConnection<()>,
}

/// A handler for a Launchpad input connection that can be polled for new messages. The actual
/// polling methods are implemented inside [MsgPollingWrapper](crate::MsgPollingWrapper). Look there
/// for documentation on how to poll messages.
pub struct InputDeviceHandlerPolling<Message> {
    _connection: MidiInputConnection<()>,
    receiver: std::sync::mpsc::Receiver<Message>,
}

impl<Message> crate::MsgPollingWrapper for InputDeviceHandlerPolling<Message> {
    type Message = Message;

    fn receiver(&self) -> &std::sync::mpsc::Receiver<Self::Message> {
        &self.receiver
    }
}

pub trait InputDevice {
    const MIDI_CONNECTION_NAME: &'static str;
    const MIDI_DEVICE_KEYWORD: &'static str;
    type Message;

    fn decode_message(timestamp: u64, data: &[u8]) -> Self::Message;

    #[must_use = "If not saved, the connection will be immediately dropped"]
    fn from_port<F>(
        midi_input: MidiInput,
        port: &MidiInputPort,
        mut user_callback: F,
    ) -> Result<InputDeviceHandler, crate::MidiError>
    where
        F: FnMut(Self::Message) + Send + 'static,
    {
        let midir_callback = move |timestamp: u64, data: &[u8], _: &mut _| {
            let msg = Self::decode_message(timestamp, data);
            (user_callback)(msg);
        };

        let connection =
            midi_input.connect(port, Self::MIDI_CONNECTION_NAME, midir_callback, ())?;

        Ok(InputDeviceHandler {
            _connection: connection,
        })
    }

    #[must_use = "If not saved, the connection will be immediately dropped"]
    fn from_port_polling(
        midi_input: MidiInput,
        port: &MidiInputPort,
    ) -> Result<InputDeviceHandlerPolling<Self::Message>, crate::MidiError>
    where
        Self::Message: Send + 'static,
    {
        let (sender, receiver) = std::sync::mpsc::channel();
        let midir_callback = move |timestamp: u64, data: &[u8], _: &mut _| {
            let msg = Self::decode_message(timestamp, data);
            // The following statement can only panic when the receiver was dropped but the
            // connection is still alive. This can't happen by accident I think, because the
            // user would have to destructure the input device handler in order to get the
            // connection and the receiver seperately, in order to drop one but not the other -
            // but if he does that it's his fault that he gets a panic /shrug
            sender
                .send(msg)
                .expect("Message receiver has hung up (this shouldn't happen)");
        };

        let connection =
            midi_input.connect(port, Self::MIDI_CONNECTION_NAME, midir_callback, ())?;

        Ok(InputDeviceHandlerPolling {
            _connection: connection,
            receiver,
        })
    }

    /// Search the midi devices and choose the first midi device matching the wanted Launchpad type.
    #[must_use = "If not saved, the connection will be immediately dropped"]
    fn guess<F>(user_callback: F) -> Result<InputDeviceHandler, crate::MidiError>
    where
        F: FnMut(Self::Message) + Send + 'static,
    {
        debug!(
            "Attempting to guess input device with keyword: '{}'",
            Self::MIDI_DEVICE_KEYWORD
        );
        let midi_input = MidiInput::new(crate::APPLICATION_NAME)?;

        let port = guess_port(&midi_input, Self::MIDI_DEVICE_KEYWORD).ok_or(
            crate::MidiError::NoPortFound {
                keyword: Self::MIDI_DEVICE_KEYWORD,
            },
        )?;
        debug!(
            "Successfully connected to input device: '{}'",
            Self::MIDI_DEVICE_KEYWORD
        );
        Self::from_port(midi_input, &port, user_callback)
    }

    /// Search the midi devices and choose the first midi device matching the wanted Launchpad type.
    #[must_use = "If not saved, the connection will be immediately dropped"]
    fn guess_polling() -> Result<InputDeviceHandlerPolling<Self::Message>, crate::MidiError>
    where
        Self::Message: Send + 'static,
    {
        debug!(
            "Attempting to guess input device (polling) with keyword: '{}'",
            Self::MIDI_DEVICE_KEYWORD
        );
        let midi_input = MidiInput::new(crate::APPLICATION_NAME)?;

        let port = guess_port(&midi_input, Self::MIDI_DEVICE_KEYWORD).ok_or(
            crate::MidiError::NoPortFound {
                keyword: Self::MIDI_DEVICE_KEYWORD,
            },
        )?;
        debug!(
            "Successfully connected to input device (polling): '{}'",
            Self::MIDI_DEVICE_KEYWORD
        );
        Self::from_port_polling(midi_input, &port)
    }
}

/// An iterator that yields canvas input messages for some user-defined time duration. For more
/// information, see [MsgPollingWrapper::iter_for]
pub struct IterFor<'a, M> {
    receiver: &'a std::sync::mpsc::Receiver<M>,
    deadline: std::time::Instant,
}

impl<M> Iterator for IterFor<'_, M> {
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {
        let now = std::time::Instant::now();

        if now >= self.deadline {
            return None;
        }

        self.receiver
            .recv_timeout(self.deadline - std::time::Instant::now())
            .ok()
    }
}

// I have no idea what I'm doing
pub trait MsgPollingWrapper {
    /// The type of message that is yielded
    type Message;

    /// Returns a [std::sync::mpsc::Receiver] that yields messages, where the type of messages is
    /// described by the associated [`Self::Message`] type
    fn receiver(&self) -> &std::sync::mpsc::Receiver<Self::Message>;

    /// Wait for a message to arrive, and return that. For a non-block variant, see
    /// [`Self::try_recv`].
    fn recv(&self) -> Self::Message {
        self.receiver()
            .recv()
            .expect("Message sender has hung up - please report a bug")
    }

    /// If there is a pending message, return that. Otherwise, return `None`.
    ///
    /// This function does not block.
    fn try_recv(&self) -> Option<Self::Message> {
        use std::sync::mpsc::TryRecvError;
        match self.receiver().try_recv() {
            Ok(msg) => Some(msg),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                panic!("Message sender has hung up - please report a bug")
            }
        }
    }

    /// Receives a single message. If no message arrives within the timespan specified by `timeout`,
    /// `None` is returned.
    fn recv_timeout(&self, timeout: std::time::Duration) -> Option<Self::Message> {
        use std::sync::mpsc::RecvTimeoutError;
        match self.receiver().recv_timeout(timeout) {
            Ok(msg) => Some(msg),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => {
                panic!("Message sender has hung up - please report a bug")
            }
        }
    }

    /// Returns an iterator over all arriving messages. The iterator will only return when the
    /// MIDI connection has been dropped.
    ///
    /// For an iteration method that doesn't block, but returns immediately when there are no more
    /// pending messages, see [`Self::iter_pending`].
    fn iter(&self) -> std::sync::mpsc::Iter<'_, Self::Message> {
        self.receiver().iter()
    }

    /// Returns an iterator over the currently pending messages. As soon as all pending messages
    /// have been iterated over, the iterator will return.
    ///
    /// For an iteration method that will block, waiting for new messages to arrive, see
    /// [`Self::iter`].
    fn iter_pending(&self) -> std::sync::mpsc::TryIter<'_, Self::Message> {
        self.receiver().try_iter()
    }

    /// Returns an iterator that yields all arriving messages for a specified amount of time. After
    /// the specified `duration` has passed, the iterator will stop and not yield any more messages.
    ///
    /// For a shorthand of this function that accepts the duration in milliseconds, see
    /// [`Self::iter_for_millis`]
    fn iter_for(&self, duration: std::time::Duration) -> IterFor<'_, Self::Message> {
        IterFor {
            receiver: self.receiver(),
            deadline: std::time::Instant::now() + duration,
        }
    }

    /// Returns an iterator that yields all arriving messages for a specified amount of time. After
    /// the specified `duration` has passed, the iterator will stop and not yield any more messages.
    ///
    /// For a more general version of this function that accepts any [std::time::Duration], see
    /// [`Self::iter_for`]
    fn iter_for_millis(&self, millis: u64) -> IterFor<'_, Self::Message> {
        self.iter_for(std::time::Duration::from_millis(millis))
    }

    /// Drain of any pending messages. This is useful on Launchpad startup - the Launchpad has the
    /// weird property that any button inputs while disconnected queue up and will all be released
    /// at the same time as soon as someone connects to it. In most cases you don't want to deal
    /// with those stale messages though - in those cases, call `drain()` after establishing the
    /// connection.
    ///
    /// This function returns the number of messages that were discarded.
    ///
    /// This is equivalent to `self.iter_pending().count()`.
    fn drain(&self) -> usize {
        self.iter_pending().count()
    }
}
