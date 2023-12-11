use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
	Io(io::Error),
	Parse,
	Eof,
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Error::Io(value)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match self {
			&Error::Io(ref err) =>
			write!(f, "{}", err),

			&Error::Parse =>
			write!(f, "Parsing error."),

			&Error::Eof =>
			write!(f, "EOF reached."),
		}
	}
}

