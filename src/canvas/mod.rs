//!	The canvas module provides incredibly useful abstractions over the low-level devices.
//! 
//! It allows you to write code that works on any Launchpad - be it the original Launchpad from
//! 2009, the Launchpad MK2, or even the 12 buttons on a Launch Control!
//! 
//! Additionally, you can chain multiple devices together as if they were a single device, using
//! `CanvasLayout`.
//! 
//! **Please look into the documentation of `Canvas`, `CanvasIterator` and `CanvasLayout` for
//! detailed documentation and examples!**

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