use thiserror::Error;

#[derive(Error, Debug)]
pub enum MidiError {
    #[error("Connecting to MIDI input port failed")]
    InputConnectError(midir::ConnectErrorKind),

    #[error("Connecting to MIDI output port failed")]
    OutputConnectError(midir::ConnectErrorKind),

    #[error("MIDI context initialization failed")]
    InitError(#[from] midir::InitError),

    #[error("MIDI Port retrieval failed")]
    PortInfoError(#[from] midir::PortInfoError),

    #[error("Sending MIDI message failed")]
    SendError(#[from] midir::SendError),

    #[error("Couldn't find a fitting port")]
    NoPortFound {
        // The keyword that was searched for
        keyword: &'static str,
    },
}

impl From<midir::ConnectError<midir::MidiInput>> for MidiError {
    fn from(error: midir::ConnectError<midir::MidiInput>) -> Self {
        Self::InputConnectError(error.kind())
    }
}

impl From<midir::ConnectError<midir::MidiOutput>> for MidiError {
    fn from(error: midir::ConnectError<midir::MidiOutput>) -> Self {
        Self::OutputConnectError(error.kind())
    }
}
