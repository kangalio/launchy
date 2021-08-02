/// An ad-hoc 2d array. This is used internally for buffering light state changes.
#[doc(hidden)] // people probably don't need this, or even _want_ to use this
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Array2d<T: Default + Copy> {
    width: u32,
    height: u32,
    vec: Vec<T>,
}

impl<T: Default + Copy> Array2d<T> {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            vec: vec![T::default(); (width * height) as usize],
        }
    }

    fn to_vec_index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }

    pub fn get(&self, x: u32, y: u32) -> Option<&T> {
        self.vec.get(self.to_vec_index(x, y)?)
    }

    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut T> {
        let i = self.to_vec_index(x, y)?;
        self.vec.get_mut(i)
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}
