use super::*;
use crate::util::Array2d;

pub struct MockCanvas {
    curr_buf: Array2d<Color>,
    new_buf: Array2d<Color>,
}

impl MockCanvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            curr_buf: Array2d::new(width, height),
            new_buf: Array2d::new(width, height),
        }
    }
}

impl crate::Canvas for MockCanvas {
    fn bounding_box(&self) -> (u32, u32) {
        (self.curr_buf.width(), self.curr_buf.height())
    }

    fn low_level_get(&self, x: u32, y: u32) -> Option<&Color> {
        self.curr_buf.get(x, y)
    }

    fn low_level_get_pending(&self, x: u32, y: u32) -> Option<&Color> {
        self.new_buf.get(x, y)
    }

    fn low_level_get_pending_mut(&mut self, x: u32, y: u32) -> Option<&mut Color> {
        self.new_buf.get_mut(x, y)
    }

    fn flush(&mut self) -> Result<(), crate::MidiError> {
        self.curr_buf = self.new_buf.clone();
        Ok(())
    }

    fn lowest_visible_brightness(&self) -> f32 {
        1.0 / 64.0
    }
}

impl_traits_for_canvas!(MockCanvas[]);
