use ndless::input::Key;
use ndless::input::{get_keys, wait_no_key_pressed};
use ndless::io;
use ndless::path::Path;
use ndless::prelude::*;
use ndless_freetype::Face;
use ndless_sdl::video::Color::RGB;
use ndless_sdl::video::Surface;

use crate::card::Flashcard;
use crate::util::BoolToggleExt;
use crate::util::DrawText;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct CardSelector<'a> {
	font: &'a Face,
	cards: &'a [Flashcard],
	starred_only: bool,
	name: String,
	starred_count: usize,
}

impl<'a> CardSelector<'a> {
	pub fn new(font: &'a Face, cards: &'a [Flashcard], path: impl AsRef<Path>) -> io::Result<Self> {
		let name = path
			.as_ref()
			.file_stem()
			.expect("failed to get file name")
			.to_string_lossy();
		let starred_count = cards.iter().filter(|card| card.star).count();
		Ok(CardSelector {
			font,
			cards,
			starred_only: false,
			name: name.into_owned(),
			starred_count,
		})
	}
	fn render(&mut self, screen: &Surface) {
		screen.fill(RGB(255, 255, 255));
		if self.cards.is_empty() {
			DrawText {
				text: "No cards in this file",
				font: self.font,
				color: RGB(69, 69, 69),
				bg: RGB(255, 255, 255),
				size: 40,
				y: 50,
				x: 5,
				kerning: true,
				wrap: false,
			}
			.draw(screen);
			screen.flip();
			return;
		}

		DrawText {
			text: &self.name,
			font: self.font,
			color: RGB(69, 69, 69),
			bg: RGB(255, 255, 255),
			size: 35,
			y: 25,
			x: 5,
			kerning: true,
			wrap: false,
		}
		.draw(screen);
		DrawText {
			text: &format!("{} terms", self.cards.len()),
			font: self.font,
			color: RGB(127, 127, 127),
			bg: RGB(255, 255, 255),
			size: 20,
			y: 50,
			x: 5,
			kerning: true,
			wrap: false,
		}
		.draw(screen);
		DrawText {
			text: &format!("{} starred terms", self.starred_count),
			font: self.font,
			color: RGB(127, 127, 127),
			bg: RGB(255, 255, 255),
			size: 20,
			y: 65,
			x: 5,
			kerning: true,
			wrap: false,
		}
		.draw(screen);
		DrawText {
			text: &format!(
				"Studying {} terms",
				if self.starred_only { "starred" } else { "all" }
			),
			font: self.font,
			color: RGB(69, 69, 69),
			bg: RGB(255, 255, 255),
			size: 30,
			y: 100,
			x: 5,
			kerning: true,
			wrap: false,
		}
		.draw(screen);
		if self.starred_count > 0 {
			DrawText {
				text: "(Press flag or . to change)",
				font: self.font,
				color: RGB(127, 127, 127),
				bg: RGB(255, 255, 255),
				size: 20,
				y: 120,
				x: 5,
				kerning: true,
				wrap: false,
			}
			.draw(screen);
		}
		DrawText {
			text: "Press enter to continue",
			font: self.font,
			color: RGB(127, 127, 127),
			bg: RGB(255, 255, 255),
			size: 20,
			y: 220,
			x: 5,
			kerning: true,
			wrap: false,
		}
		.draw(screen);
		DrawText {
			text: "Press esc to cancel",
			font: self.font,
			color: RGB(127, 127, 127),
			bg: RGB(255, 255, 255),
			size: 20,
			y: 235,
			x: 5,
			kerning: true,
			wrap: false,
		}
		.draw(screen);
		screen.flip();
	}
	pub fn run(&mut self, screen: &Surface) -> Option<bool> {
		self.render(screen);
		wait_no_key_pressed();
		loop {
			let render = {
				let keys = get_keys();
				match &keys[..] {
					&[Key::Flag] | &[Key::Period] => {
						if self.starred_count > 0 {
							self.starred_only.toggle();
						}
						true
					}
					&[Key::Enter] if !self.cards.is_empty() => break Some(self.starred_only),
					&[Key::Esc] => break None,
					_ => false,
				}
			};
			if render {
				self.render(screen);
				wait_no_key_pressed();
			}
		}
	}
}
