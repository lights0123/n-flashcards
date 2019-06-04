use csv_core::{ReadFieldResult, Reader, Writer};
use ndless::io;
use ndless::io::{BufRead, Write};
use ndless::prelude::*;

#[derive(Default, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Flashcard {
	pub a: String,
	pub b: String,
	pub star: bool,
}

#[derive(Default, Clone, Eq, PartialEq, Hash, Debug)]
struct PartialFlashcard {
	a: Option<String>,
	b: Option<String>,
	star: Option<bool>,
}

impl PartialFlashcard {
	fn fill_with_value(&mut self, val: impl AsRef<[u8]>) {
		let val = val.as_ref();
		if self.a.is_none() {
			self.a = Some(String::from_utf8_lossy(val).to_string());
		} else if self.b.is_none() {
			self.b = Some(String::from_utf8_lossy(val).to_string());
		} else if self.star.is_none() {
			self.star = Some(is_str_truthy(val));
		}
	}
}

impl From<PartialFlashcard> for Flashcard {
	fn from(part: PartialFlashcard) -> Self {
		Flashcard {
			a: part.a.unwrap_or_default(),
			b: part.b.unwrap_or_default(),
			star: part.star.unwrap_or_default(),
		}
	}
}

pub fn parse_csv(reader: impl BufRead) -> io::Result<Vec<Flashcard>> {
	let mut flashcards = vec![];
	let mut rdr = Reader::new();
	for line in reader.lines() {
		let mut line = line?;
		line.push('\n');
		let mut bytes = line.as_bytes();
		let mut card = PartialFlashcard::default();
		loop {
			let out = &mut [0; 1024];
			let (result, consumed, written) = rdr.read_field(bytes, out);
			bytes = &bytes[consumed..];
			match result {
				ReadFieldResult::InputEmpty => {}
				ReadFieldResult::OutputFull => Err(io::Error::new(
					io::ErrorKind::WriteZero,
					"The CSV entry was too large",
				))?,
				ReadFieldResult::Field { .. } => card.fill_with_value(&out[..written]),
				ReadFieldResult::End => break,
			}
		}
		if card.a.is_some() {
			flashcards.push(card.into());
		}
	}
	Ok(flashcards)
}

pub fn write_csv(writer: &mut impl Write, cards: &[Flashcard]) -> io::Result<()> {
	let out = &mut [0; 1024];
	let mut wtr = Writer::new();
	for card in cards {
		let (_, _, n) = wtr.field(card.a.as_bytes(), out);
		writer.write_all(&out[..n])?;
		let (_, n) = wtr.delimiter(out);
		writer.write_all(&out[..n])?;
		let (_, _, n) = wtr.field(card.b.as_bytes(), out);
		writer.write_all(&out[..n])?;
		if card.star {
			let (_, n) = wtr.delimiter(out);
			writer.write_all(&out[..n])?;
			let (_, _, n) = wtr.field(b"starred", out);
			writer.write_all(&out[..n])?;
		}
		let (_, n) = wtr.terminator(out);
		writer.write_all(&out[..n])?;
	}
	Ok(())
}

fn is_str_truthy(bytes: &[u8]) -> bool {
	if let Ok(phrase) = core::str::from_utf8(bytes) {
		let phrase = phrase.trim();
		phrase == "1"
			|| phrase.eq_ignore_ascii_case("true")
			|| phrase
				.get(..4)
				.map(|str| str.eq_ignore_ascii_case("star"))
				.unwrap_or(false)
	} else {
		false
	}
}
