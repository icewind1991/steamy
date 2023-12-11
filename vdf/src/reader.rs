use std::ops::Deref;
use nom::Err::Incomplete;
use crate::parser::{self, Token};
use crate::{Result as Res, Error};

/// Kinds of item.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Item {
	/// A statement, the ones starting with #.
	Statement(String),

	/// A value.
	Value(String),
}

impl Into<String> for Item {
	fn into(self) -> String {
		match self {
			Item::Statement(s) => s,
			Item::Value(s)     => s,
		}
	}
}

impl Deref for Item {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		match self {
			&Item::Statement(ref v) =>
				v,

			&Item::Value(ref v) =>
				v,
		}
	}
}

/// Reader event.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
	/// A group with the given name is starting.
	GroupStart(String),

	/// A group has ended.
	GroupEnd,

	/// An entry.
	Entry(Item, Item),

	/// EOF has been reached.
	End,
}

/// A VDF token reader.
pub struct Reader<'a> {
	buffer: &'a [u8],
}

impl<'a> From<&'a [u8]> for Reader<'a> {
	fn from(buffer: &'a [u8]) -> Self {
		Reader {
			buffer,
		}
	}
}

impl<'a> Reader<'a> {
	/// Get the next parser token without doing any copies.
	pub fn token(&mut self) -> Res<Token<'a>> {
		let (remaining, token) = match parser::next(&self.buffer) {
			Ok(res) => Ok(res),
			Err(Incomplete(_)) => Err(Error::Eof),
			Err(_) => {
				Err(Error::Parse)
			}
		}?;
		self.buffer = remaining;
		Ok(token)
	}

	pub fn non_comment_token(&mut self) -> Res<Token<'a>> {
		loop {
			match self.token()? {
				Token::Comment(_) => {
					continue;
				},
				token => return Ok(token)
			}
		}
	}

	/// Get the next event, this does copies.
	pub fn event(&mut self) -> Res<Event> {
		let key = match self.non_comment_token() {
			Err(Error::Eof) =>
				return Ok(Event::End),

			Err(err) =>
				return Err(err),

			Ok(Token::GroupEnd) =>
				return Ok(Event::GroupEnd),

			Ok(Token::GroupStart) =>
				return Err(Error::Parse),

			Ok(Token::Item(s)) =>
				Item::Value(s.into_owned()),

			Ok(Token::Statement(s)) =>
				Item::Statement(s.into_owned()),

			Ok(Token::Comment(_)) =>
				unreachable!()
		};

		let value = match self.non_comment_token() {
			Err(Error::Eof) =>
				return Ok(Event::End),

			Err(err) =>
				return Err(err),

			Ok(Token::GroupEnd) =>
				return Err(Error::Parse),

			Ok(Token::GroupStart) =>
				return Ok(Event::GroupStart(key.into())),

			Ok(Token::Item(s)) =>
				Item::Value(s.into_owned()),

			Ok(Token::Statement(s)) =>
				Item::Statement(s.into_owned()),

			Ok(Token::Comment(_)) =>
				unreachable!()
		};

		Ok(Event::Entry(key, value))
	}
}

impl<'a> Iterator for Reader<'a> {
	type Item = Event;

	fn next(&mut self) -> Option<Self::Item> {
		match self.event() {
			Ok(Event::End) =>
				None,

			Ok(event) =>
				Some(event),

			Err(..) =>
				None
		}
	}
}
