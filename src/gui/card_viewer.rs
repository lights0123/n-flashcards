use ndless::fs::OpenOptions;
use ndless::input::touchpad::touchpad_scan;
use ndless::input::Key;
use ndless::input::{get_keys, wait_no_key_pressed};
use ndless::io;
use ndless::path::PathBuf;
use ndless::prelude::*;
use ndless::thread::sleep;
use ndless::time::Duration;
use ndless_freetype::Face;
use ndless_sdl::video::Color::RGB;
use ndless_sdl::video::Surface;

use crate::assets::{STAR, STAR_FILLED};
use crate::card::Flashcard;
use crate::util::BoolToggleExt;
use crate::util::DrawText;

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
enum CardSide {
	Front,
	Back,
}

impl CardSide {
	pub fn toggle(&mut self) {
		*self = match self {
			CardSide::Front => CardSide::Back,
			CardSide::Back => CardSide::Front,
		}
	}
}

impl Default for CardSide {
	fn default() -> Self {
		CardSide::Front
	}
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct CardViewer<'a> {
	font: &'a Face,
	path: PathBuf,
	cards: &'a mut [Flashcard],
	index: usize,
	starred_only: bool,
	side: CardSide,
	default_side: CardSide,
}

impl<'a> CardViewer<'a> {
	pub fn new(
		font: &'a Face,
		path: impl Into<PathBuf>,
		cards: &'a mut [Flashcard],
		starred_only: bool,
	) -> io::Result<Self> {
		let path = path.into();
		Ok(CardViewer {
			font,
			path,
			cards,
			index: 0,
			starred_only,
			side: Default::default(),
			default_side: Default::default(),
		})
	}
	fn cards(&self) -> impl Iterator<Item = &Flashcard> {
		let starred_only = self.starred_only;
		self.cards
			.iter()
			.filter(move |card| !starred_only || card.star)
	}
	fn render(&mut self, screen: &Surface) {
		screen.fill(RGB(255, 255, 255));
		let index_str = format!("{}/{}", self.index + 1, self.cards().count());
		DrawText {
			text: &index_str,
			font: self.font,
			color: RGB(127, 127, 127),
			bg: RGB(255, 255, 255),
			size: 20,
			y: 15,
			x: 10,
			kerning: true,
			wrap: false,
		}
		.draw(screen);
		let mut cards = self.cards();
		let card = cards.nth(self.index).expect("Failed to get card");
		let text = match self.side {
			CardSide::Front => &card.a,
			CardSide::Back => &card.b,
		};
		DrawText {
			text,
			font: self.font,
			color: RGB(69, 69, 69),
			bg: RGB(255, 255, 255),
			size: 30,
			y: 45,
			x: 10,
			kerning: true,
			wrap: true,
		}
		.draw(screen);
		let star: &Surface = if card.star { &*STAR_FILLED } else { &*STAR };
		screen.blit_at(star, (screen.get_width() - star.get_width() - 4) as i16, 4);
		screen.flip();
	}
	pub fn run(&mut self, screen: &Surface) -> io::Result<()> {
		self.render(screen);
		wait_no_key_pressed();
		let mut slider_pos: Option<i16> = None;
		loop {
			let starred_only = self.starred_only;
			let render = {
				let keys = get_keys();
				let mut cards = self
					.cards
					.iter_mut()
					.filter(move |card| !starred_only || card.star);
				match &keys[..] {
					&[Key::Flag] | &[Key::Period] => {
						let card = cards.nth(self.index).expect(concat!(
							"failed to get card [",
							file!(),
							"]",
							" ",
							line!(),
							":",
							column!()
						));
						card.star.toggle();
						if starred_only {
							self.index = self
								.index
								.checked_sub(1)
								.unwrap_or_else(|| cards.count() - 1);
							if dbg!(self.cards().next()).is_none() {
								break;
							}
						}
						true
					}
					&[Key::Click] | &[Key::Enter] => {
						self.side.toggle();
						true
					}
					&[Key::Right] => {
						self.index += 1;
						if self.index >= cards.count() {
							self.index = 0;
						}
						self.side = self.default_side;
						true
					}
					&[Key::Left] => {
						self.index = self
							.index
							.checked_sub(1)
							.unwrap_or_else(|| cards.count() - 1);
						self.side = self.default_side;
						true
					}
					&[Key::Esc] => break,
					_ => false,
				}
			};
			if render {
				self.render(screen);
				wait_no_key_pressed();
				slider_pos = None;
			} else if ndless::hw::has_touchpad() {
				let report = touchpad_scan().expect("Failed to read from touchpad");
				if report.proximity < 47 {
					slider_pos = None;
					continue;
				}
				let pos = (i32::from(report.x) - i32::from(core::u16::MAX)) as i16;
				if let Some(prev) = slider_pos {
					if pos - prev != 0 {
						let count = self
							.cards
							.iter_mut()
							.filter(move |card| !starred_only || card.star)
							.count();
						let skip = pos - prev;
						let render = if skip > 20 {
							self.index += 1;
							if self.index >= count {
								self.index = 0;
								self.side = self.default_side;
							}
							true
						} else if skip < -20 {
							self.index = self.index.checked_sub(1).unwrap_or_else(|| count - 1);
							self.side = self.default_side;
							true
						} else {
							false
						};
						if render {
							self.render(screen);
							sleep(Duration::from_millis(20));
						}
					}
				}
				slider_pos = Some(pos);
			}
		}
		let mut file = OpenOptions::new().write(true).open(&self.path)?;
		crate::card::write_csv(&mut file, &self.cards[..])?;
		Ok(())
	}
}
