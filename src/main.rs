#![no_std]
#![no_main]
#![allow(clippy::match_ref_pats)]
#[macro_use]
extern crate lazy_static;
extern crate ndless_handler;

use ndless::fs::OpenOptions;
use ndless::io;
use ndless::io::BufReader;

use ndless::prelude::*;
use ndless_sdl::video::{SurfaceFlag, VideoFlag};

use gui::card_selector::CardSelector;
use gui::card_viewer::CardViewer;

use crate::gui::file_picker::FilePicker;

mod assets;
mod card;
mod gui;
mod owned_surface;
mod util;

#[entry]
fn main() -> io::Result<()> {
	let library = ndless_freetype::Library::init().expect("failed to create freetype library");
	let face = library
		.new_static_face(assets::ROBOTO, 0)
		.expect("failed to load font");
	ndless_sdl::init(&[ndless_sdl::InitFlag::Video]);
	ndless_sdl::mouse::set_cursor_visible(false);
	let screen = match ndless_sdl::video::set_video_mode(
		320,
		240,
		if ndless::hw::has_colors() { 16 } else { 8 },
		&[SurfaceFlag::SWSurface],
		&[VideoFlag::NoFrame],
	) {
		Ok(screen) => screen,
		Err(err) => panic!("failed to set video mode: {}", err),
	};
	while let Some(path) = FilePicker::new(&face, "/documents/")?.run(&screen) {
		let file = OpenOptions::new().read(true).open(&path)?;
		let mut cards = crate::card::parse_csv(BufReader::new(file))?;
		let mut card_selector = CardSelector::new(&face, &cards, &path)?;
		if let Some(starred_only) = card_selector.run(&screen) {
			let mut card_viewer = CardViewer::new(&face, path, &mut cards, starred_only)?;
			card_viewer.run(&screen)?;
		}
	}
	ndless_sdl::quit();
	Ok(())
}
