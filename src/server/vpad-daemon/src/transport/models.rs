use std::{
    error::Error,
    fmt::{Display, Formatter},
    net::IpAddr,
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
pub enum Axis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    Button(Button),
    LeftStick(Axis),
    RightStick(Axis),
    Trigger(Trigger),
}

impl InputType {
    pub const BUTTON: u8 = 0;
    pub const LEFT_STICK: u8 = 1;
    pub const RIGHT_STICK: u8 = 2;
    pub const TRIGGER: u8 = 3;
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputSignalError {
    InvalidInputType(u8),
    InvalidButtonCode(u8),
    InvalidAxisCode(u8),
    InvalidTriggerCode(u8),
}

impl Display for InputSignalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInputType(byte) => write!(formatter, "Invalid input type: {byte}"),
            Self::InvalidButtonCode(byte) => write!(formatter, "Invalid button code: {byte}"),
            Self::InvalidAxisCode(byte) => write!(formatter, "Invalid axis code: {byte}"),
            Self::InvalidTriggerCode(byte) => write!(formatter, "Invalid trigger code: {byte}"),
        }
    }
}

impl Error for InputSignalError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientId {
    Network(IpAddr),
}

/// Represents an input action, similar to `input_event` structure of Linux.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputSignal {
    pub itype: InputType,
    pub value: i16,
}

impl InputSignal {
    /// The size of the structure.
    pub const SIZE: usize = size_of::<InputSignal>();
}

impl TryFrom<&[u8; Self::SIZE]> for InputSignal {
    type Error = InputSignalError;

    /// Converts a buffer of bytes into an InputSignal.
    /// 1. First byte: Type of the input.
    /// 2. Second byte: Code of the input.
    /// 3. From third to fourth byte: The value of input action.
    fn try_from(buffer: &[u8; Self::SIZE]) -> Result<Self, InputSignalError> {
        let type_byte = buffer[0];
        let code_byte = buffer[1];
        let value_bytes = buffer[2..Self::SIZE].try_into().unwrap();
        let value = i16::from_be_bytes(value_bytes);
        let itype = match type_byte {
            InputType::BUTTON => match code_byte {
                0 => InputType::Button(Button::South),
                1 => InputType::Button(Button::North),
                2 => InputType::Button(Button::West),
                3 => InputType::Button(Button::East),
                4 => InputType::Button(Button::LeftBumper),
                5 => InputType::Button(Button::RightBumper),
                6 => InputType::Button(Button::DPadDown),
                7 => InputType::Button(Button::DPadUp),
                8 => InputType::Button(Button::DPadLeft),
                9 => InputType::Button(Button::DPadRight),
                10 => InputType::Button(Button::LeftStick),
                11 => InputType::Button(Button::RightStick),
                12 => InputType::Button(Button::Select),
                13 => InputType::Button(Button::Start),
                _ => return Err(InputSignalError::InvalidButtonCode(code_byte)),
            },
            InputType::LEFT_STICK => match code_byte {
                0 => InputType::LeftStick(Axis::X),
                1 => InputType::LeftStick(Axis::Y),
                _ => return Err(InputSignalError::InvalidAxisCode(code_byte)),
            },
            InputType::RIGHT_STICK => match code_byte {
                0 => InputType::RightStick(Axis::X),
                1 => InputType::RightStick(Axis::Y),
                _ => return Err(InputSignalError::InvalidAxisCode(code_byte)),
            },
            InputType::TRIGGER => match code_byte {
                0 => InputType::Trigger(Trigger::Left),
                1 => InputType::Trigger(Trigger::Right),
                _ => return Err(InputSignalError::InvalidTriggerCode(code_byte)),
            },
            _ => return Err(InputSignalError::InvalidInputType(code_byte)),
        };

        Ok(InputSignal { itype, value })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputMessage {
    pub client_id: ClientId,
    pub signal: InputSignal,
}
