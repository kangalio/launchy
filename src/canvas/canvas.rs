use super::*;

/// A trait that abstracts over the specifics of a Launchpad and allows generic access and
/// manipulation of a Launchpad's LEDs.
///
/// # How do you use a canvas?
///
/// [`Canvas`]es work by first accumulating LED changes, and finally flushing all LED state changes
/// in an efficient manner by calling `.flush()`.
///
/// Every [`Canvas`] maintains two buffers: the unflushed one, and the edited one. Therefore, you
/// can access both the unflushed and the buffered state of the button, using `get_old` and `get`,
/// respectively.
///
/// Example:
/// ```no_run
/// # use launchy::{Canvas, Color};
/// fn light_white(canvas: &mut impl Canvas) {
///     // Iterate through all buttons in the canvas. See the documentation on [`CanvasIterator`] for
///     // more info
///     for pad in canvas.iter() {
///         canvas[pad] = Color::WHITE;
///     }
/// }
///
/// // The above function doesn't take a specific low-level object like Output or
/// // Output. Instead it utilizes Canvas, so you can call it with _all_ devices!
///
/// // Light a connected Launchpad S and Launchpad Mk2 completely white
/// light_white(&mut launchy::s::Canvas::guess(|_| {})?);
/// light_white(&mut launchy::mk2::Canvas::guess(|_| {})?);
/// # Ok::<(), launchy::MidiError>(())
/// ```
pub trait Canvas:
    std::ops::Index<Pad, Output = Color> + std::ops::IndexMut<Pad, Output = Color>
{
    // These are the methods that _need_ to be implemented by the implementor

    /// The width and height of the smallest rectangle that still fully encapsulates the shape of
    /// this canvas
    ///
    /// ```
    /// # use launchy::Canvas as _;
    /// # /*
    /// let mut canvas = launchy::mk2::Canvas::guess(|_| {})?;
    /// # */ let mut canvas = launchy::MockCanvas::new(9, 9);
    ///
    /// assert_eq!(canvas.bounding_box(), (9, 9));
    /// ```
    fn bounding_box(&self) -> (u32, u32);

    /// Returns a reference to the currently displayed color at the given position
    fn low_level_get(&self, x: u32, y: u32) -> Option<&Color>;
    /// Returns a reference to the in-buffer/unflushed color at the given position
    fn low_level_get_pending(&self, x: u32, y: u32) -> Option<&Color>;
    /// Returns a mutable reference to the in-buffer/unflushed color at the given position
    fn low_level_get_pending_mut(&mut self, x: u32, y: u32) -> Option<&mut Color>;

    /// Flush the accumulated changes to the underlying device
    ///
    /// ```
    /// # use launchy::{Pad, Color, Canvas as _};
    /// # let mut canvas = launchy::MockCanvas::new(9, 9);
    ///
    /// canvas[Pad { x: 0, y: 0 }] = Color::RED;
    /// canvas[Pad { x: 1, y: 0 }] = Color::GREEN;
    /// canvas[Pad { x: 2, y: 0 }] = Color::RED;
    /// canvas[Pad { x: 3, y: 0 }] = Color::GREEN;
    ///
    /// // The changes are only transmitted when they are flushed
    /// canvas.flush()?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    fn flush(&mut self) -> Result<(), crate::MidiError>;
    /// The lowest visible brightness on this canvas. Used to calibrate brightness across
    /// Launchpads; users of the library probably don't need to worry about this
    fn lowest_visible_brightness(&self) -> f32;

    // These are defaut implementations that you get for free

    /// Returns the currently displayed color at the given position, or None if out of bounds
    ///
    /// ```
    /// # use launchy::{Pad, Color, Canvas as _};
    /// # let mut canvas = launchy::MockCanvas::new(9, 9);
    /// canvas[Pad { x: 5, y: 5 }] = Color::RED;
    ///
    /// assert_eq!(canvas.get(Pad { x: 5, y: 5 }), Some(Color::BLACK));
    /// canvas.flush()?;
    /// assert_eq!(canvas.get(Pad { x: 5, y: 5 }), Some(Color::RED));
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    ///
    /// Use `canvas[pad]` for a simpler, panicking version
    fn get(&self, pad: Pad) -> Option<Color> {
        let (x, y) = pad.to_u32()?;
        self.low_level_get(x, y).copied()
    }

    /// Returns the buffered/unflushed color at the given position, or None if out of bounds
    ///
    /// ```
    /// # use launchy::{Pad, Color, Canvas as _};
    /// # let mut canvas = launchy::MockCanvas::new(9, 9);
    ///
    /// assert_eq!(canvas.get_pending(Pad { x: 5, y: 5 }), Some(Color::BLACK));
    /// canvas[Pad { x: 5, y: 5 }] = Color::RED;
    /// assert_eq!(canvas.get_pending(Pad { x: 5, y: 5 }), Some(Color::RED));
    ///
    /// canvas.flush()?;
    /// assert_eq!(canvas.get_pending(Pad { x: 5, y: 5 }), Some(Color::RED)); // didn't change
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    fn get_pending(&self, pad: Pad) -> Option<Color> {
        let (x, y) = pad.to_u32()?;
        self.low_level_get_pending(x, y).copied()
    }

    /// Sets the color at the given position. Returns None if out of bounds
    ///
    /// Use `canvas[pad] = color` for a simpler, panicking version
    fn set(&mut self, pad: Pad, color: Color) -> Option<()> {
        let (x, y) = pad.to_u32()?;
        *self.low_level_get_pending_mut(x, y)? = color;

        Some(())
    }

    /// An iterator over the buttons of a given Canvas. Create an iterator by calling `.iter()` on a
    /// [`Canvas`].
    ///
    /// This iterator returns [`Pad`]s, which are a view on a single button on the canvas. See
    /// the documentation on [`Pad`] for more information.
    ///
    /// For example to light the entire canvas white:
    /// ```
    /// # use launchy::{Color, Pad, Canvas as _};
    /// # let mut canvas = launchy::MockCanvas::new(9, 9);
    /// for pad in canvas.iter() {
    ///     canvas[pad] = Color::WHITE;
    /// }
    /// canvas.flush()?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    ///
    /// Or, if you want to move the entire contents of the canvas one pixel to the right:
    /// ```
    /// # use launchy::{Color, Pad, Canvas as _};
    /// # let mut canvas = launchy::MockCanvas::new(9, 9);
    /// for pad in canvas.iter() {
    ///     // Move this pad's color to the right
    ///     canvas.set(pad.right(1), canvas[pad]);
    /// }
    /// canvas.flush()?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    fn iter(&self) -> CanvasIterator {
        CanvasIterator::new(self)
    }

    /// Toggles the button at the specified coordinate with the given color.
    ///
    /// Light the specified button with the given color, except if the button is already lit with
    /// that color; in which case the button is turned off.
    ///
    /// For example, if you were to make a paint program for the Launchpad MK2 where you can toggle
    /// pixels by pressing:
    /// ```no_run
    /// # use launchy::{Color, CanvasMessage, Canvas as _, MsgPollingWrapper as _};
    /// let (mut canvas, poller) = launchy::mk2::Canvas::guess_polling()?;
    ///
    /// for msg in poller.iter() {
    ///     if msg.is_press() {
    ///         canvas.toggle(msg.pad(), Color::WHITE);
    ///         canvas.flush()?;
    ///     }
    /// }
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    fn toggle(&mut self, pad: Pad, color: Color) -> Option<()> {
        let (x, y) = pad.to_u32()?;
        let current_color = self.low_level_get_pending_mut(x, y)?;

        if *current_color == color {
            *current_color = Color::BLACK;
        } else {
            *current_color = color;
        }

        Some(())
    }

    /// Clear the entire canvas by setting all buttons to black.
    ///
    /// ```
    /// # use launchy::{Color, Pad, Canvas as _};
    /// # let mut canvas = launchy::MockCanvas::new(9, 9);
    /// // light a square in the top left for one second, then clear it
    ///
    /// canvas[Pad { x: 0, y: 0 }] = Color::MAGENTA;
    /// canvas[Pad { x: 0, y: 1 }] = Color::MAGENTA;
    /// canvas[Pad { x: 1, y: 0 }] = Color::MAGENTA;
    /// canvas[Pad { x: 1, y: 1 }] = Color::MAGENTA;
    /// canvas.flush()?;
    ///
    /// # /*
    /// std::thread::sleep_ms(1000);
    /// # */
    ///
    /// canvas.clear();
    /// canvas.flush()?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    fn clear(&mut self)
    where
        Self: Sized,
    {
        for pad in self.iter() {
            self[pad] = Color::BLACK;
        }
    }

    /// See [`PaddingCanvas`] for more details
    fn into_padded(self) -> PaddingCanvas<Self>
    where
        Self: Sized,
    {
        PaddingCanvas::from(self)
    }

    fn is_valid(&self, pad: Pad) -> bool {
        match pad.to_u32() {
            Some((x, y)) => self.low_level_get(x, y).is_some(),
            None => false,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum CanvasMessage {
    Press { x: u32, y: u32 },
    Release { x: u32, y: u32 },
}

impl CanvasMessage {
    /// Retrieves the x coordinate of this message, no matter if this is a press or a release
    /// message
    pub fn x(&self) -> u32 {
        match *self {
            Self::Press { x, y: _ } => x,
            Self::Release { x, y: _ } => x,
        }
    }

    /// Retrieves the y coordinate of this message, no matter if this is a press or a release
    /// message
    pub fn y(&self) -> u32 {
        match *self {
            Self::Press { x: _, y } => y,
            Self::Release { x: _, y } => y,
        }
    }

    pub fn pad(&self) -> Pad {
        Pad {
            x: self.x() as i32,
            y: self.y() as i32,
        }
    }

    /// Returns whether this is a press message
    pub fn is_press(&self) -> bool {
        matches!(self, Self::Press { .. })
    }

    /// Returns whether this is a release message
    pub fn is_release(&self) -> bool {
        matches!(self, Self::Release { .. })
    }
}
