use core::convert::{TryFrom, TryInto};

use ndless::ffi::OsStrExt;
use ndless::fs::DirEntry;
use ndless::input::Key;
use ndless::input::{get_keys, wait_no_key_pressed};
use ndless::io;
use ndless::path::PathBuf;
use ndless::prelude::*;
use ndless_freetype::Face;
use ndless_sdl::video::Color::RGB;
use ndless_sdl::video::{Color, Surface};
use ndless_sdl::Rect;

use crate::assets::FILE;
use crate::assets::FOLDER;

use crate::util::DrawText;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Copy)]
enum EntryType {
	File,
	Directory,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct FsEntry {
	pub path: PathBuf,
	pub name: String,
	pub file_type: EntryType,
}

impl TryFrom<DirEntry> for FsEntry {
	type Error = io::Error;

	fn try_from(entry: DirEntry) -> io::Result<Self> {
		let path = entry.path();
		let file_type = if path.is_file() {
			EntryType::File
		} else {
			EntryType::Directory
		};
		if file_type == EntryType::File
			&& !path
				.file_name()
				.ok_or(io::ErrorKind::InvalidInput)?
				.as_bytes()
				.ends_with(b".csv.tns")
		{
			return Err(io::ErrorKind::InvalidInput.into());
		}
		let name = path
			.file_stem()
			.ok_or(io::ErrorKind::NotFound)?
			.to_string_lossy()
			.into_owned();
		Ok(FsEntry {
			path,
			name,
			file_type,
		})
	}
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct FilePicker<'a> {
	font: &'a Face,
	path: PathBuf,
	files: Vec<FsEntry>,
	index: usize,
	top_index: usize,
}

const MAX_FILES_ON_SCREEN: usize = 8;
const SELECTED_COLOR: Color = RGB(54, 123, 240);

impl<'a> FilePicker<'a> {
	pub fn new(font: &'a Face, path: impl Into<PathBuf>) -> io::Result<Self> {
		let path = path.into();
		let mut files: Vec<FsEntry> = path
			.read_dir()?
			.filter_map(|entry| entry.ok())
			.filter_map(|entry| entry.try_into().ok())
			.collect();
		files.sort_unstable_by(|a, b| a.name.cmp(&b.name));
		Ok(FilePicker {
			font,
			path,
			files,
			index: 0,
			top_index: 0,
		})
	}
	fn render(&self, screen: &Surface) {
		screen.fill(RGB(255, 255, 255));
		if self.files.is_empty() {
			DrawText {
				text: "No files",
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

		screen.fill_rect(
			Some(Rect {
				x: 0,
				y: 0,
				w: screen.get_width(),
				h: 28,
			}),
			RGB(54, 50, 80),
		);
		let text = &self
			.path
			.strip_prefix("/documents")
			.unwrap_or(&self.path)
			.to_string_lossy();
		DrawText {
			text: &format!("/{}", text),
			font: self.font,
			color: RGB(255, 255, 255),
			bg: RGB(54, 50, 80),
			size: 25,
			y: 20,
			x: 5,
			kerning: true,
			wrap: false,
		}
		.draw(screen);
		let on_screen = (MAX_FILES_ON_SCREEN).min(self.files.len());
		let mut y = 30;
		for (i, file) in self
			.files
			.iter()
			.enumerate()
			.skip(self.top_index)
			.take(on_screen)
		{
			let bg = if i == self.index {
				screen.fill_rect(
					Some(Rect {
						x: 0,
						y: y - 2,
						w: screen.get_width(),
						h: 25,
					}),
					SELECTED_COLOR,
				);
				SELECTED_COLOR
			} else {
				RGB(255, 255, 255)
			};
			let icon: &Surface = if file.file_type == EntryType::Directory {
				&*FOLDER
			} else {
				&*FILE
			};
			screen.blit_at(icon, 5, y);
			DrawText {
				text: &file.name,
				font: self.font,
				color: if i == self.index {
					RGB(255, 255, 255)
				} else {
					RGB(69, 69, 69)
				},
				bg,
				size: 20,
				y: y as usize + 15,
				x: 27,
				kerning: true,
				wrap: false,
			}
			.draw(screen);
			y += 25;
		}
		screen.flip();
	}
	pub fn run(mut self, screen: &Surface) -> Option<PathBuf> {
		self.render(screen);
		wait_no_key_pressed();
		loop {
			let render = {
				let keys = get_keys();
				match &keys[..] {
					&[Key::Down] => {
						if self.index < self.files.len() - 1 {
							self.index += 1;
							if self.index >= self.top_index + MAX_FILES_ON_SCREEN {
								self.top_index += 1;
							}
						} else {
							self.index = 0;
							self.top_index = 0;
						}
						true
					}
					&[Key::Up] => {
						if self.index > 0 {
							self.index -= 1;
							if self.index < self.top_index {
								self.top_index -= 1;
							}
						} else {
							self.index = self.files.len() - 1;
							self.top_index =
								self.index.checked_sub(MAX_FILES_ON_SCREEN - 1).unwrap_or(0);
						}

						true
					}
					&[Key::Enter] if !self.files.is_empty() => {
						let file = &mut self.files[self.index];
						if file.file_type == EntryType::File {
							break Some(core::mem::replace(&mut file.path, PathBuf::new()));
						} else {
							let picker = FilePicker::new(self.font, &file.path)
								.ok()
								.and_then(|picker| picker.run(screen));
							if let Some(path) = picker {
								break Some(path);
							}
						}
						true
					}
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
