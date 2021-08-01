// use super::*;

/// A 2d point that represents a single pad on a grid of pads. For convenience, the coordinates can
/// be negative.
///
/// [`Pad`] implements various mathematical operator traits for the `(i32, i32)` tuple. Therefore
/// it's possible move a pad around by adding or subtracting (x, y) coordinate tuples:
///
/// ```rust
/// let pad = Pad { x: 3, y: 6 };
///
/// assert_eq!(pad + (4, 4), Pad { x: 7, y: 10 });
/// assert_eq!(pad - (4, 4), Pad { x: -1, y: 2 });
/// ```
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Hash)]
pub struct Pad {
    pub x: i32,
    pub y: i32,
}

impl Pad {
    /// Return a copy of this [`Pad`], moved upwards by a certain number of steps
    pub fn up(self, steps: i32) -> Self {
        Self {
            x: self.x,
            y: self.y - steps,
        }
    }

    /// Return a copy of this [`Pad`], moved downwards by a certain number of steps
    pub fn down(self, steps: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + steps,
        }
    }

    /// Return a copy of this [`Pad`], moved left by a certain number of steps
    pub fn left(self, steps: i32) -> Self {
        Self {
            x: self.x - steps,
            y: self.y,
        }
    }

    /// Return a copy of this [`Pad`], moved right by a certain number of steps
    pub fn right(self, steps: i32) -> Self {
        Self {
            x: self.x + steps,
            y: self.y,
        }
    }

    /// Returns an array of the four surrounding neighbors of this pad
    pub fn neighbors_4(self) -> [Self; 4] {
        [self.up(1), self.right(1), self.down(1), self.left(1)]
    }

    /// Returns an array of the four surrounding neighbors of this pad, plus itself
    pub fn neighbors_5(self) -> [Self; 5] {
        [self, self.up(1), self.right(1), self.down(1), self.left(1)]
    }

    /// Returns an array of the eight surrounding neighbors of this pad
    pub fn neighbors_8(self) -> [Self; 8] {
        [
            self.up(1),
            self.up(1).right(1),
            self.right(1),
            self.right(1).down(1),
            self.down(1),
            self.down(1).left(1),
            self.left(1),
            self.left(1).up(1),
        ]
    }

    /// Returns an array of the eight surrounding neighbors of this pad, plus itself
    pub fn neighbors_9(self) -> [Self; 9] {
        [
            self,
            self.up(1),
            self.up(1).right(1),
            self.right(1),
            self.right(1).down(1),
            self.down(1),
            self.down(1).left(1),
            self.left(1),
            self.left(1).up(1),
        ]
    }
}

impl std::ops::Add<(i32, i32)> for Pad {
    type Output = Self;

    fn add(self, offset: (i32, i32)) -> Self {
        let (x_offset, y_offset) = offset;
        Self {
            x: self.x + x_offset,
            y: self.y + y_offset,
        }
    }
}

impl std::ops::AddAssign<(i32, i32)> for Pad {
    fn add_assign(&mut self, offset: (i32, i32)) {
        let (x_offset, y_offset) = offset;
        self.x += x_offset;
        self.y += y_offset;
    }
}

impl std::ops::Sub<(i32, i32)> for Pad {
    type Output = Self;

    fn sub(self, offset: (i32, i32)) -> Self {
        let (x_offset, y_offset) = offset;
        Self {
            x: self.x - x_offset,
            y: self.y - y_offset,
        }
    }
}

impl std::ops::SubAssign<(i32, i32)> for Pad {
    fn sub_assign(&mut self, offset: (i32, i32)) {
        let (x_offset, y_offset) = offset;
        self.x -= x_offset;
        self.y -= y_offset;
    }
}
