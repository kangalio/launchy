use super::*;
use crate::util::Array2d;

/// A canvas wrapper that fills holes and other irregularities to provide a perfectly rectangular
/// grid.
///
/// Launchpads traditionally have non-rectangular shapes:
/// - Most Launchpads are _almost_ 9x9 - just one pixel is missing
/// - The Launchpad Pro is _almost_ 10x10, except each corner has a pixel missing
/// - The Launch Control has an even _more_ irregular placement if the four control buttons are
///   included
///
/// Or, even worse, when having a multi-Launchpad setup using CanvasLayout - that's even less
/// rectangular; there will be gaps and empty spaces all over the place.
///
/// Sometimes it's a pain to deal with those irregular shapes. For that reason, this struct provides
/// a [`Canvas`] wrapper that inserts 'virtual pixels' into all the gaps so that you can work with
/// the Launchpad as if it were a perfectly rectangular grid.
///
/// Example:
/// ```no_run
/// # use launchy::{Pad, Canvas as _};
/// // Create the base canvas
/// let mut canvas = launchy::mk2::Canvas::guess(|msg| {})?;
///
/// // Wrap the holey canvas in `PaddingCanvas`
/// let mut canvas = launchy::PaddingCanvas::from(canvas);
///
/// // Now you can fearlessly work with the canvas as if it's a rectangle
/// for y in 0..9 {
///     for x in 0..9 {
///         canvas[Pad { x, y }] = launchy::Color::WHITE;
///     }
/// }
/// # Ok::<(), launchy::MidiError>(())
/// ```
pub struct PaddingCanvas<C: Canvas> {
    inner: C,
    curr_buf: Array2d<Color>,
    new_buf: Array2d<Color>,
}

impl<C: Canvas> PaddingCanvas<C> {
    /// Wrap the given canvas in a PaddingCanvas
    pub fn from(inner: C) -> Self {
        let (width, height) = inner.bounding_box();

        Self {
            inner,
            curr_buf: Array2d::new(width, height),
            new_buf: Array2d::new(width, height),
        }
    }
}

impl<C: Canvas> Canvas for PaddingCanvas<C> {
    fn bounding_box(&self) -> (u32, u32) {
        self.inner.bounding_box()
    }
    fn lowest_visible_brightness(&self) -> f32 {
        self.inner.lowest_visible_brightness()
    }

    fn low_level_get(&self, x: u32, y: u32) -> Option<&Color> {
        if let Some(color) = self.inner.low_level_get(x, y) {
            Some(color)
        } else {
            self.curr_buf.get(x, y)
        }
    }

    fn low_level_get_pending(&self, x: u32, y: u32) -> Option<&Color> {
        if let Some(color) = self.inner.low_level_get_pending(x, y) {
            Some(color)
        } else {
            self.new_buf.get(x, y)
        }
    }

    fn low_level_get_pending_mut(&mut self, x: u32, y: u32) -> Option<&mut Color> {
        if let Some(color) = self.inner.low_level_get_pending_mut(x, y) {
            Some(color)
        } else {
            self.new_buf.get_mut(x, y)
        }
    }

    fn flush(&mut self) -> Result<(), crate::MidiError> {
        self.curr_buf = self.new_buf.clone();
        self.inner.flush()
    }
}

impl_traits_for_canvas!(PaddingCanvas[C: Canvas]);
