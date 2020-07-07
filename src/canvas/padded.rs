use super::*;
use crate::util::Array2d;

/// Launchpads traditionally have non-rectangular shapes:
/// - Most Launchpads are _almost_ 9x9 - just one pixel is missing
/// - The Launchpad Pro is _almost_ 10x10, except each corner has a pixel missing
/// - The Launch Control has an even _more_ irregular placement if the four control buttons are
/// 	included
/// 
/// Or, even worse, when having a multi-Launchpad setup using CanvasLayout - that's even less
/// rectangular; there will be gaps and empty spaces all over the place.
/// 
/// Sometimes it's a pain to deal with those irregular shapes. For that reason, this struct provides
/// a `Canvas` wrapper that inserts 'virtual pixels' into all the gaps so that you can work with the
/// Launchpad as if it were a perfectly rectangular grid.
/// 
/// Example:
/// ```rust
/// // Create the base canvas
/// let mut canvas = launchy::mk2::Canvas::guess(|msg| {});
/// 
/// // Wrap the holey canvas in `PaddingCanvas`
/// let mut canvas = launchy::PaddingCanvas::from(canvas);
/// 
/// // Now you can fearlessly work with the canvas as if it's a rectangle
/// for y in 0..9 {
/// 	for x in 0..9 {
/// 		canvas.set(x, y, launchy::Color::WHITE);
/// 	}
/// }
/// ```
pub struct PaddingCanvas<C: Canvas> {
	inner: C,
	curr_buf: Array2d<Color>,
	new_buf: Array2d<Color>,
}

impl<C: Canvas> PaddingCanvas<C> {
	/// Wrap the given canvas in a PaddingCanvas
	pub fn from(inner: C) -> Self {
		let width = inner.bounding_box_width() as usize;
		let height = inner.bounding_box_height() as usize;

		Self {
			inner,
			curr_buf: Array2d::new(width, height),
			new_buf: Array2d::new(width, height),
		}
	}
}

impl<C: Canvas> Canvas for PaddingCanvas<C> {
    fn bounding_box_width(&self) -> u32 {
        self.inner.bounding_box_width()
	}
	
    fn bounding_box_height(&self) -> u32 {
        self.inner.bounding_box_height()
	}
	
    fn is_valid(&self, x: u32, y: u32) -> bool {
        x < self.bounding_box_width() && y < self.bounding_box_height()
	}
	
    fn get_unchecked(&self, x: u32, y: u32) -> Color {
        if self.inner.is_valid(x, y) {
			self.inner.get_unchecked(x, y)
		} else {
			self.new_buf.get(x as usize, y as usize)
		}
	}
	
    fn set_unchecked(&mut self, x: u32, y: u32, color: Color) {
		if self.inner.is_valid(x, y) {
			self.inner.set_unchecked(x, y, color);
		} else {
			self.new_buf.set(x as usize, y as usize, color);
		}
	}
	
    fn get_old_unchecked(&self, x: u32, y: u32) -> Color {
        if self.inner.is_valid(x, y) {
			self.inner.get_old_unchecked(x, y)
		} else {
			self.curr_buf.get(x as usize, y as usize)
		}
	}
	
    fn flush(&mut self) -> anyhow::Result<()> {
		self.curr_buf = self.new_buf.clone();
		self.inner.flush()
    }
}

impl_traits_for_canvas!(<C: Canvas>, PaddingCanvas);