use core::ops::{Deref, DerefMut};

use ndless_sdl::video::Surface;

pub struct OwnedSurface {
	surface: Surface,
}

impl OwnedSurface {
	pub fn new(surface: Surface) -> Option<Self> {
		if surface.owned {
			Some(OwnedSurface { surface })
		} else {
			None
		}
	}
}

impl Deref for OwnedSurface {
	type Target = Surface;
	fn deref(&self) -> &Self::Target {
		&self.surface
	}
}

impl DerefMut for OwnedSurface {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.surface
	}
}

// Safe because ndless is single-threaded
unsafe impl Send for OwnedSurface {}
unsafe impl Sync for OwnedSurface {}
