use std::str;
use std::borrow::Cow;
use nom::branch::alt;
use nom::bytes::streaming::{take_while, tag, is_not, escaped, take};
use nom::{error::Error, IResult, Err::*, Needed};
use nom::error::ErrorKind;
use nom::sequence::delimited;

/// Parser token.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token<'a> {
	/// A group is starting.
	GroupStart,

	/// A group is ending.
	GroupEnd,

	/// An enclosed or bare item.
	Item(Cow<'a, str>),

	/// An enclosed or bare statement.
	Statement(Cow<'a, str>),
}

fn string(buffer: &[u8]) -> Result<Cow<str>, Error<&[u8]>> {
	let bytes = buffer;
	if buffer.iter().any(|&b| b == b'\\') {
		let mut buffer = buffer.iter().copied();
		let mut string = Vec::with_capacity(buffer.len());

		while let Some(byte) = buffer.next() {
			if byte == b'\\' {
				match buffer.next() {
					Some(b'\\') => string.push(b'\\'),
					Some(b'n')  => string.push(b'\n'),
					Some(b't')  => string.push(b'\t'),
					Some(b'r')  => string.push(b'\r'),
					Some(b'"')  => string.push(b'"'),
					Some(byte)  => string.extend_from_slice(&[b'\\', byte]),
					None        => break
				}
			}
			else {
				string.push(byte);
			}
		}

		match String::from_utf8(string) {
			Err(err) => Err(Error::new(bytes, ErrorKind::Verify)),
			Ok(str)  => Ok(str.into())
		}
	}
	else {
		Ok(str::from_utf8(buffer).map_err(|_|Error::new(bytes, ErrorKind::Verify))?.into())
	}
}

fn whitespace(input: &[u8]) -> IResult<&[u8], &[u8]> {
	match take_while(|b: u8| b.is_ascii_whitespace())(input) {
		Ok(res) => Ok(res),
		// end of input
		Err(Incomplete(Needed::Size(size))) if size.get() == 1 => Ok((&[][..], input)),
		Err(e) => Err(e)
	}
}

pub fn next(input: &[u8]) -> IResult<&[u8], Token> {
	let (input, _) = whitespace(input)?;
	let (input, value) = alt((open, close, bare, enclosed))(input)?;
	let (input, _) = whitespace(input)?;
	Ok((input, value))
}

fn open(input: &[u8]) -> IResult<&[u8], Token> {
	let (input, _) = tag(b"{")(input)?;
	Ok((input, Token::GroupStart))
}

fn close(input: &[u8]) -> IResult<&[u8], Token> {
	let (input, _) = tag(b"}")(input)?;
	Ok((input, Token::GroupEnd))
}

fn empty_item(input: &[u8]) -> IResult<&[u8], Token> {
	let (input, _) = tag("\"\"")(input)?;
	Ok((input, Token::Item(Cow::Borrowed(""))))
}

fn bare(input: &[u8]) -> IResult<&[u8], Token> {
	alt((bare_statement, bare_item))(input)
}

/// like `is_not`, but returning `Ok` instead of `Incomplete` if the full input is consumed
fn is_not_alt<'a>(arr: &'static str) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
	move |input: &[u8]| {
		match is_not(arr)(input) {
			Ok(res) => Ok(res),
			// end of input
			Err(Incomplete(Needed::Size(size))) if size.get() == 1 => Ok((&[][..], input)),
			Err(e) => Err(e)
		}
	}
}

fn bare_statement(input: &[u8]) -> IResult<&[u8], Token> {
	let (input, _) = tag(b"#")(input)?;
	let (input, value) = is_not_alt(" \t\n\r{}\"")(input)?;
	let value = string(value).map_err(Error)?;
	Ok((input, Token::Statement(value)))
}

fn bare_item(input: &[u8]) -> IResult<&[u8], Token> {
	let (input, value) = is_not_alt(" \t\n\r{}\"")(input)?;
	let value = string(value).map_err(Error)?;
	Ok((input, Token::Item(value)))
}

fn enclosed(input: &[u8]) -> IResult<&[u8], Token> {
	alt((enclosed_statement, enclosed_item, empty_item))(input)
}

fn enclosed_content(input: &[u8]) -> IResult<&[u8], &[u8]> {
	escaped(is_not_alt("\"\\"), '\\', take(1usize))(input)
}

fn enclosed_statement(input: &[u8]) -> IResult<&[u8], Token> {
	let (input, value) = delimited(tag(b"\""), |input| {
		let (input, _) = tag(b"#")(input)?;
		enclosed_content(input)
	}, tag(b"\""))(input)?;
	let value = string(value).map_err(Error)?;
	Ok((input, Token::Statement(value)))
}

fn enclosed_item(input: &[u8]) -> IResult<&[u8], Token> {
	let (input, value) = delimited(tag(b"\""), enclosed_content, tag(b"\""))(input)?;
	let value = string(value).map_err(Error)?;
	Ok((input, Token::Item(value)))
}

#[cfg(test)]
mod tests {
	use super::Token;

	#[test]
	fn next() {
		assert_eq!(super::next(b"test"), Ok((&b""[..], Token::Item("test".into()))));
		assert_eq!(super::next(b"\"test\""), Ok((&b""[..], Token::Item("test".into()))));
		assert_eq!(super::next(b"\"\""), Ok((&b""[..], Token::Item("".into()))));
		assert_eq!(super::next(b"#test"), Ok((&b""[..], Token::Statement("test".into()))));
		assert_eq!(super::next(b"\"#test\""), Ok((&b""[..], Token::Statement("test".into()))));
		assert_eq!(super::next(b"{"), Ok((&b""[..], Token::GroupStart)));
		assert_eq!(super::next(b"}"), Ok((&b""[..], Token::GroupEnd)));
	}

	#[test]
	fn bare() {
		assert_eq!(super::bare(b"test"), Ok((&b""[..], Token::Item("test".into()))));
		assert_eq!(super::bare(b"#test"), Ok((&b""[..], Token::Statement("test".into()))));

		assert_eq!(super::bare(b"lol wut"), Ok((&b" wut"[..], Token::Item("lol".into()))));
		assert_eq!(super::bare(b"#lol wut"), Ok((&b" wut"[..], Token::Statement("lol".into()))));

		assert_eq!(super::bare(b"lol{"), Ok((&b"{"[..], Token::Item("lol".into()))));
		assert_eq!(super::bare(b"#lol{"), Ok((&b"{"[..], Token::Statement("lol".into()))));

		assert_eq!(super::bare(b"lol}"), Ok((&b"}"[..], Token::Item("lol".into()))));
		assert_eq!(super::bare(b"#lol}"), Ok((&b"}"[..], Token::Statement("lol".into()))));
	}

	#[test]
	fn enclosed() {
		assert_eq!(super::enclosed(b"\"test\""), Ok((&b""[..], Token::Item("test".into()))));
		assert_eq!(super::enclosed(b"\"#test\""), Ok((&b""[..], Token::Statement("test".into()))));

		assert_eq!(super::enclosed(b"\"te\\\"st\""), Ok((&b""[..], Token::Item("te\"st".into()))));
		assert_eq!(super::enclosed(b"\"te\\st\""), Ok((&b""[..], Token::Item("te\\st".into()))));
		assert_eq!(super::enclosed(b"\"#te\\\"st\""), Ok((&b""[..], Token::Statement("te\"st".into()))));
	}

	#[test]
	fn open() {
		assert_eq!(super::open(b"{"), Ok((&b""[..], Token::GroupStart)));
	}

	#[test]
	fn close() {
		assert_eq!(super::close(b"}"), Ok((&b""[..], Token::GroupEnd)));
	}
}
