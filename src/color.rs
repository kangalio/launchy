/// A simple float-based color struct. Each component should lie in 0..=1, but it can also be
/// outside that range. If outside, it will be clipped for some operations
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Color {
	pub r: f32,
	pub g: f32,
	pub b: f32
}

impl Color {
	pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0 };
	pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0 };
	pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0 };
	pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0 };
	pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0 };
	pub const CYAN: Color = Color { r: 0.0, g: 1.0, b: 1.0 };
	pub const MAGENTA: Color = Color { r: 1.0, g: 0.0, b: 1.0 };
	pub const YELLOW: Color = Color { r: 1.0, g: 1.0, b: 0.0 };

	pub fn new(r: f32, g: f32, b: f32) -> Self {
		return Self { r, g, b };
	}

	/// Creates a color from a hue, starting at 0.0 (red) and ending at 1.0 (red). You can pass in
	/// any number though, cuz it's "circular": 0.0 == 1.0 == -1.0 == 2.0 ...
	pub fn from_hue(hue: f32) -> Self {
		return match hue * 6.0 {
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
				return Self::from_hue(hue);
			}
		}
	}

	/// Return a tuple of color components scaled from 0..=1 to 0..range by doing
	/// `(component * range).floor().min(range - 1).max(0)` on every component
	pub fn quantize(&self, range: u8) -> (u8, u8, u8) {
		return (
			((self.r * range as f32) as u8).min(range - 1).max(0),
			((self.g * range as f32) as u8).min(range - 1).max(0),
			((self.b * range as f32) as u8).min(range - 1).max(0),
		);
	}
	
	// TODO: decide if we really want this, and if we do, whether this is a good implementation
	pub fn quantize_human(&self, range: u8) -> (u8, u8, u8) {
		let quantize = |v: f32| ((v * range as f32).ceil() as u8).min(range - 1).max(0);
		return (quantize(self.r), quantize(self.g), quantize(self.b));
	}
}

impl std::ops::Mul<f32> for Color {
	type Output = Self;

	fn mul(self, multiplier: f32) -> Self::Output {
		return Self {
			r: self.r * multiplier,
			g: self.g * multiplier,
			b: self.b * multiplier,
		};
	}
}

impl std::ops::Div<f32> for Color {
	type Output = Self;

	fn div(self, multiplier: f32) -> Self::Output {
		return Self {
			r: self.r / multiplier,
			g: self.g / multiplier,
			b: self.b / multiplier,
		};
	}
}

impl std::ops::Add for Color {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		return Self {
			r: self.r + other.r,
			g: self.g + other.g,
			b: self.b + other.b,
		};
	}
}

impl std::ops::Sub for Color {
	type Output = Self;
	
	fn sub(self, other: Self) -> Self {
		return Self {
			r: self.r - other.r,
			g: self.g - other.g,
			b: self.b - other.b,
		};
	}
}

impl std::ops::Add<f32> for Color {
	type Output = Self;

	fn add(self, addend: f32) -> Self {
		return Self {
			r: self.r + addend,
			g: self.g + addend,
			b: self.b + addend,
		};
	}
}

impl std::ops::Sub<f32> for Color {
	type Output = Self;

	fn sub(self, subtrand /* or something like that */: f32) -> Self {
		return Self {
			r: self.r - subtrand,
			g: self.g - subtrand,
			b: self.b - subtrand,
		};
	}
}