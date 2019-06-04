use crate::owned_surface::OwnedSurface;

lazy_static! {
	pub static ref STAR: OwnedSurface = {
		OwnedSurface::new(
			ndless_sdl::image::load_mem_gif(include_bytes!("assets/star.gif"))
				.expect("failed to load star.gif"),
		)
		.unwrap()
	};
	pub static ref STAR_FILLED: OwnedSurface = {
		OwnedSurface::new(
			ndless_sdl::image::load_mem_gif(include_bytes!("assets/star_filled.gif"))
				.expect("failed to load star_filled.gif"),
		)
		.unwrap()
	};
	pub static ref FOLDER: OwnedSurface = {
		OwnedSurface::new(
			ndless_sdl::image::load_mem_gif(include_bytes!("assets/folder.gif"))
				.expect("failed to load folder.gif"),
		)
		.unwrap()
	};
	pub static ref FILE: OwnedSurface = {
		OwnedSurface::new(
			ndless_sdl::image::load_mem_gif(include_bytes!("assets/file.gif"))
				.expect("failed to load file.gif"),
		)
		.unwrap()
	};
}

/// See https://stackoverflow.com/a/44440992 for how the reduced file was generated
/// Also turn off "PS Glyph Names" in FontForge
pub const ROBOTO: &[u8] = include_bytes!("assets/Roboto-Light-reduced.ttf");
