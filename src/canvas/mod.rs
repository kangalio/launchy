//!    The canvas module provides incredibly useful abstractions over the low-level devices.
//!
//! It allows you to write code that works on any Launchpad - be it the original Launchpad from
//! 2009, the Launchpad MK2, or even the 12 buttons on a Launch Control!
//!
//! Additionally, you can chain multiple devices together as if they were a single device, using
//! [`CanvasLayout`].
//!
//! **Please look into the documentation of [`Canvas`], [`CanvasIterator`] and [`CanvasLayout`] for
//! detailed documentation and examples!**

// I want to have certain traits implemented for all [`Canvas`]es. Unfortunately, I can't use
// blanket implementations for this purpose (orphan rules forbid it). For that reason, I have to
// manually duplicate the trait implementations for each Canvas implementor. For that, I made this
// macro.
macro_rules! impl_traits_for_canvas {
    (<$($a:tt $(: $b:tt)?),+>, $i:ident) => {
        impl<$($a $(: $b)?),+> std::ops::Index<Pad> for $i<$($a),+> {
            type Output = Color;

            fn index(&self, pad: Pad) -> &Color {
                self.get_ref(pad).expect("Pad coordinates out of bounds")
            }
        }

        impl<$($a $(: $b)?),+> std::ops::IndexMut<Pad> for $i<$($a),+> {
            fn index_mut(&mut self, pad: Pad) -> &mut Color {
                self.get_mut(pad).expect("Pad coordinates out of bounds")
            }
        }

        #[cfg(feature = "embedded-graphics-support")]
        mod eg {
            pub use embedded_graphics::{prelude::*, DrawTarget, pixelcolor::{Rgb888, RgbColor}};
        }

        #[cfg(feature = "embedded-graphics-support")]
        impl<$($a $(: $b)?),+> eg::DrawTarget<eg::Rgb888> for $i<$($a),+> {
            type Error = ();

            fn draw_pixel(&mut self, pixel: eg::Pixel<eg::Rgb888>) -> Result<(), ()> {
                let eg::Pixel(coord, color) = pixel;

                // discard any potential out of bounds errors. that's just how it's done in
                // embedded-graphics world
                let _ = self.set(Pad { x: coord.x, y: coord.y }, color.into());

                Ok(())
            }

            fn size(&self) -> eg::Size {
                eg::Size::new(self.bounding_box_width(), self.bounding_box_height())
            }
        }
    }
}

mod iterator;
pub use iterator::*;

mod layout;
pub use layout::*;

mod generic;
pub use generic::*;

mod color;
pub use color::*;

// the outer module is for "everything canvas", and the inner module is the core Canvas
// functionality. There is reason behind this module inception
#[allow(clippy::module_inception)]
mod canvas;
pub use canvas::*;

mod padded;
pub use padded::*;

mod pad;
pub use pad::*;
