pub enum ThreadCommand {
    ProcessorComplete,
}

pub const PAGE_SIZE: usize = 33;

pub enum Command {
    Handshake { status: u8 },
    RMS { left: u8, right: u8 },
}

pub enum CommandParseError {
    CommandByteError,
    HandshakeStatusError,
    RMSValueError(u8),
    UndefinedCommand(u8),
}

impl Command {
    pub fn to_data(&self) -> Vec<u8> {
        match self {
            Command::Handshake { status } => vec![self.into(), *status],
            Command::RMS { left, right } => vec![self.into(), *left, *right],
        }
    }
}

impl From<&Command> for u8 {
    fn from(val: &Command) -> Self {
        match val {
            Command::Handshake { .. } => 0x01,
            Command::RMS { .. } => 0x02,
        }
    }
}

impl TryFrom<&[u8]> for Command {
    type Error = CommandParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let command_index = value.first().ok_or(Self::Error::CommandByteError)?;
        match command_index {
            0x01 => Ok(Command::Handshake {
                status: *value.get(1).ok_or(Self::Error::HandshakeStatusError)?,
            }),
            0x02 => Ok(Command::RMS {
                left: *value.get(1).ok_or(Self::Error::RMSValueError(0))?,
                right: *value.get(2).ok_or(Self::Error::RMSValueError(1))?,
            }),
            _ => Err(CommandParseError::UndefinedCommand(*command_index)),
        }
    }
}

pub struct Protocol {}

impl Protocol {
    pub fn new() -> Protocol {
        Self {}
    }

    fn header(&self) -> [u8; 4] {
        [0, b'k', b'b', b'm']
    }

    pub fn prepare_command(&self, command: &Command) -> [u8; PAGE_SIZE] {
        let mut data: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
        let header = self.header();
        let (header_chunk, data_chunk) = data.split_at_mut(header.len());
        header_chunk.copy_from_slice(&header);
        let command_data = command.to_data();
        for (index, c) in command_data.iter().enumerate() {
            data_chunk[index] = *c;
        }
        data
    }
}

impl Default for Protocol {
    fn default() -> Self {
        Self::new()
    }
}
