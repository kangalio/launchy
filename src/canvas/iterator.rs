use super::*;

/// An iterator over the buttons of a given Canvas. Create an iterator by calling `.iter()` on a
/// [`Canvas`].
///
/// For more information, see [`Canvas::iter`].
pub struct CanvasIterator(std::vec::IntoIter<Pad>);

impl CanvasIterator {
    pub(crate) fn new<C: Canvas + ?Sized>(canvas: &C) -> Self {
        let (bb_width, bb_height) = canvas.bounding_box();

        let mut coordinates = Vec::with_capacity((bb_width * bb_height) as usize);
        for y in 0..bb_height {
            for x in 0..bb_width {
                if canvas.low_level_get(x, y).is_some() {
                    coordinates.push(Pad {
                        x: x as i32,
                        y: y as i32,
                    });
                }
            }
        }

        CanvasIterator(coordinates.into_iter())
    }
}

impl Iterator for CanvasIterator {
    type Item = Pad;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
