use std::{
    error::Error,
    fmt::{Display, Formatter},
    net::SocketAddr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    South,
    North,
    West,
    East,
    LeftBumper,
    RightBumper,
    DPadDown,
    DPadUp,
    DPadLeft,
    DPadRight,
    LeftStick,
    RightStick,
    Select,
    Start,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSignalError {
    InvalidFormat(u8),
    InvalidInputType(u8),
    InvalidButtonCode(u8),
    InvalidSideCode(u8),
}

impl Display for InputSignalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(byte) => {
                write!(formatter, "Invalid format of bytes for input type: {byte}")
            }
            Self::InvalidInputType(byte) => write!(formatter, "Invalid input type: {byte}"),
            Self::InvalidButtonCode(byte) => write!(formatter, "Invalid button code: {byte}"),
            Self::InvalidSideCode(byte) => write!(formatter, "Invalid side code: {byte}"),
        }
    }
}

impl Error for InputSignalError {}

/// Represents an input action, similar to `input_event` structure of Linux.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSignal {
    Button {
        code: Button,
        pressed: u8,
    },
    Stick {
        side: Side,
        x_value: i16,
        y_value: i16,
    },
    Trigger {
        side: Side,
        value: u8,
    },
}

impl InputSignal {
    pub const BUTTON: u8 = 0;
    pub const STICK: u8 = 1;
    pub const TRIGGER: u8 = 2;

    fn try_button_from(input_type: u8, buffer: &[u8]) -> Result<Self, InputSignalError> {
        if buffer.len() != 2 {
            return Err(InputSignalError::InvalidFormat(input_type));
        }

        let code_byte = buffer[0];
        let code = match code_byte {
            0 => Button::South,
            1 => Button::North,
            2 => Button::West,
            3 => Button::East,
            4 => Button::LeftBumper,
            5 => Button::RightBumper,
            6 => Button::DPadDown,
            7 => Button::DPadUp,
            8 => Button::DPadLeft,
            9 => Button::DPadRight,
            10 => Button::LeftStick,
            11 => Button::RightStick,
            12 => Button::Select,
            13 => Button::Start,
            _ => return Err(InputSignalError::InvalidButtonCode(code_byte)),
        };
        Ok(InputSignal::Button {
            code,
            pressed: buffer[1],
        })
    }

    fn try_stick_from(input_type: u8, buffer: &[u8]) -> Result<Self, InputSignalError> {
        if buffer.len() != 5 {
            return Err(InputSignalError::InvalidFormat(input_type));
        }

        let code_byte = buffer[0];
        let x_bytes = match buffer[1..3].try_into() {
            Ok(bytes) => bytes,
            Err(_) => return Err(InputSignalError::InvalidFormat(input_type)),
        };
        let y_bytes = match buffer[3..5].try_into() {
            Ok(bytes) => bytes,
            Err(_) => return Err(InputSignalError::InvalidFormat(input_type)),
        };
        let side = match code_byte {
            0 => Side::Left,
            1 => Side::Right,
            _ => return Err(InputSignalError::InvalidSideCode(input_type)),
        };
        Ok(InputSignal::Stick {
            side,
            x_value: i16::from_be_bytes(x_bytes),
            y_value: i16::from_be_bytes(y_bytes),
        })
    }

    fn try_trigger_from(input_type: u8, buffer: &[u8]) -> Result<Self, InputSignalError> {
        if buffer.len() != 2 {
            return Err(InputSignalError::InvalidFormat(input_type));
        }

        let code_byte = buffer[0];
        let side = match code_byte {
            0 => Side::Left,
            1 => Side::Right,
            _ => return Err(InputSignalError::InvalidSideCode(code_byte)),
        };
        Ok(InputSignal::Trigger {
            side,
            value: buffer[1],
        })
    }
}

impl TryFrom<&[u8]> for InputSignal {
    type Error = InputSignalError;

    /// Converts a buffer of bytes into an InputSignal.
    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        let type_byte = buffer[0];
        match type_byte {
            InputSignal::BUTTON => Self::try_button_from(type_byte, &buffer[1..3]),
            InputSignal::STICK => Self::try_stick_from(type_byte, &buffer[1..6]),
            InputSignal::TRIGGER => Self::try_trigger_from(type_byte, &buffer[1..3]),
            _ => Err(InputSignalError::InvalidInputType(type_byte)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMessage {
    Connect,
    Disconnect,
}

impl TryFrom<u8> for ConnectionMessage {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ConnectionMessage::Connect),
            1 => Ok(ConnectionMessage::Disconnect),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientId {
    Network(SocketAddr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageError {
    InvalidType,
    InvalidConnectionMsg,
    InvalidInputSignal(InputSignalError),
}

impl Display for MessageError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidType => write!(formatter, "Invalid message type"),
            Self::InvalidConnectionMsg => write!(formatter, "Invalid connection message"),
            Self::InvalidInputSignal(error) => write!(formatter, "{error}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientMessage {
    Connection(ConnectionMessage),
    Input(InputSignal),
}

impl ClientMessage {
    /// Max buffer size: 1 (msg_type) + 1 (input_type) + 1 (code) + 2 (X axis) + 2 (Y axis) = 7
    pub const MAX_SIZE: usize = 7;
    pub const CONNECTION: u8 = 0;
    pub const INPUT: u8 = 1;
}

impl TryFrom<&[u8; Self::MAX_SIZE]> for ClientMessage {
    type Error = MessageError;

    fn try_from(buffer: &[u8; Self::MAX_SIZE]) -> Result<Self, Self::Error> {
        let type_byte = buffer[0];
        match type_byte {
            Self::CONNECTION => match ConnectionMessage::try_from(buffer[1]) {
                Ok(conn_msg) => Ok(Self::Connection(conn_msg)),
                Err(_) => return Err(MessageError::InvalidConnectionMsg),
            },
            Self::INPUT => match InputSignal::try_from(&buffer[1..Self::MAX_SIZE]) {
                Ok(signal) => Ok(Self::Input(signal)),
                Err(error) => return Err(MessageError::InvalidInputSignal(error)),
            },
            _ => Err(MessageError::InvalidType),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientPacket {
    pub client_id: ClientId,
    pub message: ClientMessage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ServerMessage {
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerPacket<T> {
    pub client_id: T,
    pub message: ServerMessage,
}
