#[derive(Debug)]
pub enum MidiError {
    InputConnectError(midir::ConnectError<()>),
    OutputConnectError(midir::ConnectError<()>),
    InitError(midir::InitError),
    PortInfoError(midir::PortInfoError),
    SendError(midir::SendError),
    NoPortFound {
        // The keyword that was searched for
        keyword: &'static str,
    },
}

impl std::fmt::Display for MidiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputConnectError(_) => f.write_str("connecting to MIDI input port failed"),
            Self::OutputConnectError(_) => f.write_str("connecting to MIDI output port failed"),
            Self::InitError(_) => f.write_str("MIDI context initialization failed"),
            Self::PortInfoError(_) => f.write_str("MIDI Port retrieval failed"),
            Self::SendError(_) => f.write_str("sending MIDI message failed"),
            Self::NoPortFound { keyword } => write!(f, "couldn't find a port for {:?}", keyword),
        }
    }
}

impl std::error::Error for MidiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InputConnectError(e) => Some(e),
            Self::OutputConnectError(e) => Some(e),
            Self::InitError(e) => Some(e),
            Self::PortInfoError(e) => Some(e),
            Self::SendError(e) => Some(e),
            Self::NoPortFound { keyword: _ } => None,
        }
    }
}

impl From<midir::ConnectError<midir::MidiInput>> for MidiError {
    fn from(e: midir::ConnectError<midir::MidiInput>) -> Self {
        // Strip contained MidiInput from error because it's not Sync
        Self::InputConnectError(midir::ConnectError::new(e.kind(), ()))
    }
}

impl From<midir::ConnectError<midir::MidiOutput>> for MidiError {
    fn from(e: midir::ConnectError<midir::MidiOutput>) -> Self {
        // Strip contained MidiOutput from error because it's not Sync
        Self::OutputConnectError(midir::ConnectError::new(e.kind(), ()))
    }
}

impl From<midir::InitError> for MidiError {
    fn from(e: midir::InitError) -> Self {
        Self::InitError(e)
    }
}

impl From<midir::PortInfoError> for MidiError {
    fn from(e: midir::PortInfoError) -> Self {
        Self::PortInfoError(e)
    }
}

impl From<midir::SendError> for MidiError {
    fn from(e: midir::SendError) -> Self {
        Self::SendError(e)
    }
}
