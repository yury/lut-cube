#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    InvalidSize(std::num::ParseIntError),
    InvalidInputRange(String),
    InvalidRgb(String),
    UnexpectedInputRange(String),
    CubeIsFull,
    UnexpectedData,
    Eof,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::InvalidSize(value)
    }
}
