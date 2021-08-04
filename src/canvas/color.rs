/// A simple float-based color struct. Each component should lie in 0..=1, but it can also be
/// outside that range. If outside, it will be clipped for some operations
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
    };
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
    };
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
    };
    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
    };

    /// Create a new color from the given red, green, and blue components
    ///
    /// Examples:
    /// ```
    /// # use launchy::Color;
    /// let lime = Color::new(0.75, 1.0, 0.0);
    /// let beige = Color::new(0.96, 0.96, 0.86);
    /// ```
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// Creates a color from a hue, starting at 0.0 (red) and ending at 1.0 (red). You can pass in
    /// any number though, because the cycle repeats (think the `x` in `sin(x)`)
    ///
    /// ```
    /// # use launchy::Color;
    /// let red = Color::from_hue(0.0);
    /// let orange = Color::from_hue(0.1);
    /// let greenish_yellow = Color::from_hue(0.2);
    /// let green = Color::from_hue(0.3);
    /// let cyan = Color::from_hue(0.4);
    /// let light_blue = Color::from_hue(0.5);
    /// let blue = Color::from_hue(0.6);
    /// let purple = Color::from_hue(0.7);
    /// let light_pink = Color::from_hue(0.8);
    /// let strong_pink = Color::from_hue(0.9);
    /// ```
    pub fn from_hue(hue: f32) -> Self {
        match hue * 6.0 {
            hue if (0.0..1.0).contains(&hue) => Self::new(1.0, hue, 0.0), // red -> yellow
            hue if (1.0..2.0).contains(&hue) => Self::new(2.0 - hue, 1.0, 0.0), // yellow -> green
            hue if (2.0..3.0).contains(&hue) => Self::new(0.0, 1.0, hue - 2.0), // green -> cyan
            hue if (3.0..4.0).contains(&hue) => Self::new(0.0, 4.0 - hue, 1.0), // cyan -> blue
            hue if (4.0..5.0).contains(&hue) => Self::new(hue - 4.0, 0.0, 1.0), // blue -> magenta
            hue if (5.0..6.0).contains(&hue) => Self::new(1.0, 0.0, 6.0 - hue), // magenta -> red
            _ => {
                // calculate hue % 1 and then stick the modulo-ed value in
                let hue = hue.fract();
                let hue = if hue < 0.0 { 1.0 + hue } else { hue };
                Self::from_hue(hue)
            }
        }
    }

    /// Util function that smoothly interpolates between the following 'keyframes':
    /// - 0.00 => green
    /// - 0.25 => yellow
    /// - 0.50 => red
    /// - 0.75 => yellow
    /// - 1.00 => green
    ///
    /// and then the cycle continues.
    ///
    /// This function is useful to create a smooth cycling gradient of colors on non-RGB devices
    /// such as the Launchpad S.
    pub fn red_green_color(hue: f32) -> Self {
        let a = |x| {
            if x < 0.25 {
                4.0 * x
            } else if x >= 0.75 {
                4.0 - 4.0 * x
            } else {
                1.0
            }
        };

        let r = a(hue.fract());
        let g = a((hue + 0.5).fract());
        Self::new(r, g, 0.0)
    }

    /// Clamp to a range of 0..=1.
    ///
    /// If any component is below 0, it is brought up to 0. If any component is above 1, every
    /// component will be multiplied by a certain value so that every component will be at most 1.
    ///
    /// This algorithm ensures that the color hue stays the same, no matter the brightness.
    ///
    /// ```rust
    /// # use launchy::Color;
    /// assert_eq!(
    ///     launchy::Color::new(-1.0, 1.0, 3.0).clamp(),
    ///     launchy::Color::new(0.0, 1.0/3.0, 1.0),
    /// );
    /// ```
    pub fn clamp(self) -> Self {
        let Self { r, g, b } = self;

        let highest_component = r.max(g).max(b);
        let multiplier = if highest_component > 1.0 {
            1.0 / highest_component
        } else {
            1.0
        };

        let r = f32::clamp(r * multiplier, 0.0, 1.0);
        let g = f32::clamp(g * multiplier, 0.0, 1.0);
        let b = f32::clamp(b * multiplier, 0.0, 1.0);

        Self { r, g, b }
    }

    /// Return a tuple of color components scaled from 0..=1 to 0..range using the same algorithm
    /// as in [`Self::clamp`].
    ///
    /// This function is used by the Canvas implementation of the Launchpads to downscale the
    /// high-precision [`Color`]s to their respective color width. For example the Launchpad S only
    /// supports four levels of brightness for its red and green component, respectively. Therefore,
    /// the Launchpad S calls `.quantize(4)` on a given [`Color`] to derive how that color should be
    /// represented on the Launchpad S LEDs.
    pub fn quantize(self, range: u8) -> (u8, u8, u8) {
        let Self { r, g, b } = self.clamp();

        let quantize_component = |c| u8::min((c * range as f32) as u8, range - 1);
        (
            quantize_component(r),
            quantize_component(g),
            quantize_component(b),
        )
    }

    /// Mix two colors together. The proportion of the second color is specified by
    /// `proportion_of_other`.
    ///
    /// Examples:
    /// ```
    /// # use launchy::Color;
    /// let very_dark_red = Color::RED.mix(Color::BLACK, 0.9);
    /// let orange = Color::RED.mix(Color::YELLOW, 0.5);
    /// let dark_brown = Color::RED.mix(Color::YELLOW, 0.5).mix(Color::BLACK, 0.7);
    /// ```
    pub fn mix(self, other: Color, proportion_of_other: f32) -> Color {
        other * proportion_of_other + self * (1.0 - proportion_of_other)
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Self;

    fn mul(self, multiplier: f32) -> Self::Output {
        Self {
            r: self.r * multiplier,
            g: self.g * multiplier,
            b: self.b * multiplier,
        }
    }
}

impl std::ops::Div<f32> for Color {
    type Output = Self;

    fn div(self, multiplier: f32) -> Self::Output {
        Self {
            r: self.r / multiplier,
            g: self.g / multiplier,
            b: self.b / multiplier,
        }
    }
}

impl std::ops::Add for Color {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl std::ops::Sub for Color {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
        }
    }
}

impl std::ops::Add<f32> for Color {
    type Output = Self;

    fn add(self, addend: f32) -> Self {
        Self {
            r: self.r + addend,
            g: self.g + addend,
            b: self.b + addend,
        }
    }
}

impl std::ops::Sub<f32> for Color {
    type Output = Self;

    fn sub(self, subtrand /* or something like that */: f32) -> Self {
        Self {
            r: self.r - subtrand,
            g: self.g - subtrand,
            b: self.b - subtrand,
        }
    }
}

impl std::ops::Neg for Color {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            r: -self.r,
            g: -self.g,
            b: -self.b,
        }
    }
}

impl std::iter::Sum for Color {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Color::BLACK, |a, b| a + b)
    }
}

#[cfg(feature = "embedded-graphics")]
impl From<Color> for embedded_graphics::pixelcolor::Rgb888 {
    fn from(color: Color) -> Self {
        let (r, g, b) = color.quantize(256);
        Self::new(r, g, b)
    }
}

#[cfg(feature = "embedded-graphics")]
impl From<embedded_graphics::pixelcolor::Rgb888> for Color {
    fn from(color: embedded_graphics::pixelcolor::Rgb888) -> Self {
        use embedded_graphics::pixelcolor::RgbColor;

        Color::new(
            (color.r() as f32 + 0.5) / 256.0,
            (color.g() as f32 + 0.5) / 256.0,
            (color.b() as f32 + 0.5) / 256.0,
        )
    }
}
