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
                let (x, y) = pad.to_u32().expect("Pad coordinates out of bounds");
                self.low_level_get(x, y).expect("Pad coordinates out of bounds")
            }
        }

        impl<$($a $(: $b)?),+> std::ops::IndexMut<Pad> for $i<$($a),+> {
            fn index_mut(&mut self, pad: Pad) -> &mut Color {
                let (x, y) = pad.to_u32().expect("Pad coordinates out of bounds");
                self.low_level_get_pending_mut(x, y).expect("Pad coordinates out of bounds")
            }
        }

        #[cfg(feature = "embedded-graphics")]
        mod eg {
            pub use embedded_graphics::{
                prelude::*,
                draw_target::DrawTarget,
                geometry::Dimensions,
                pixelcolor::{Rgb888, RgbColor},
                primitives::rectangle::Rectangle,
            };
        }

        #[cfg(feature = "embedded-graphics")]
        impl<$($a $(: $b)?),+> eg::Dimensions for $i<$($a),+> {
            fn bounding_box(&self) -> eg::Rectangle {
                eg::Rectangle::new(
                    eg::Point::new(0, 0),
                    eg::Size::from(Canvas::bounding_box(self)),
                )
            }
        }

        #[cfg(feature = "embedded-graphics")]
        impl<$($a $(: $b)?),+> eg::DrawTarget for $i<$($a),+> {
            type Color = eg::Rgb888;
            type Error = std::convert::Infallible;

            fn draw_iter<I: IntoIterator<Item = eg::Pixel<Self::Color>>>(
                &mut self,
                pixels: I,
            ) -> Result<(), std::convert::Infallible> {
                for eg::Pixel(coord, color) in pixels.into_iter() {
                    // discard any potential out of bounds errors. that's just how it's done in
                    // embedded-graphics world
                    let _ = self.set(Pad { x: coord.x, y: coord.y }, color.into());
                }

                Ok(())
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
