use std::path::Path;

mod error;
pub use error::Error;

pub type Result<T> = ::std::result::Result<T, Error>;

pub mod reader;
pub use reader::Reader;

pub mod entry;
pub use entry::{Table, Entry, Statement, Value};

pub mod parser;

pub fn parse(buffer: &[u8]) -> Result<Entry> {
	let mut reader = Reader::from(buffer);
	Ok(Table::load(&mut reader)?.into())
}

/// Load a table from the given path.
pub fn load<P: AsRef<Path>>(path: P) -> Result<Entry> {
	let buffer = std::fs::read(path)?;
	parse(&buffer)
}
