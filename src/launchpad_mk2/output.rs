use midir::MidiOutputConnection;

use super::Button;
use crate::OutputDevice;


/// The Launchpad Mk2 has two different ways to represent color. You can either use one of the 128
/// built-in palette colors, or you can create a custom color with custom rgb components.
/// Why would you choose the palette colors when you can just create your required colors yourself?
/// Well some operations on the Mk2 only support palette colors. Besides, sending palette color midi
/// messages is simply faster. Therefore you should aim to use the palette colors when possible.

/// A color from the Mk2 color palette. See the "Launchpad MK2 Programmers Reference Manual"
/// to see the palette, or [see here](http://launchpaddr.com/mk2palette/).
/// 
/// Everywhere where a PaletteColor is expected as a funcion argument, you can also directly pass
/// in the palette index and call `.into()` on it. Example:
/// ```
/// // This:
/// output.light_all(PaletteColor::new(92));
/// // can also be written as:
/// output.light_all(92.into());
/// ```
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct PaletteColor {
	pub(crate) id: u8,
}

impl PaletteColor {
	pub fn is_valid(&self) -> bool {
		return self.id <= 127;
	}

	pub fn new(id: u8) -> Self {
		let self_ = Self { id };
		assert!(self_.is_valid());
		return self_;
	}

	pub fn id(&self) -> u8 { self.id }
	pub fn set_id(&mut self, id: u8) { self.id = id }
}

impl From<u8> for PaletteColor {
	fn from(id: u8) -> Self {
		return Self::new(id);
	}
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
/// An RGB color. Each component may only go up to 63
pub struct RgbColor {
	r: u8,
	g: u8,
	b: u8,
}

impl RgbColor {
	/// Create a new RgbColor from the individual component values
	pub fn new(r: u8, g: u8, b: u8) -> Self {
		let self_ = Self { r, g, b };
		assert!(self_.is_valid());
		return self_;
	}

	/// Check whether the rgb color is valid - each component may only go up to 63.
	pub fn is_valid(&self) -> bool {
		return self.r <= 63 && self.g <= 63 && self.b <= 63;
	}

	pub fn red(&self) -> u8 { self.r }
	pub fn green(&self) -> u8 { self.g }
	pub fn blue(&self) -> u8 { self.b }
	pub fn set_red(&mut self, r: u8) { assert!(r <= 63); self.r = r }
	pub fn set_green(&mut self, g: u8) { assert!(g <= 63); self.g = g }
	pub fn set_blue(&mut self, b: u8) { assert!(b <= 63); self.b = b }
}

impl PaletteColor {
	// These are some commonly used colors as palette colors. I don't have Rgb colors as constants
	// because in the case of rgb colors you can just make your required colors yourself

	// Basic colors, the top row
	pub const BLACK: PaletteColor = Self { id: 0 };
	pub const DARK_GRAY: PaletteColor = Self { id: 1 };
	pub const LIGHT_GRAY: PaletteColor = Self { id: 2 };
	pub const WHITE: PaletteColor = Self { id: 3 };

	// Third column from the right
	pub const RED: PaletteColor = Self { id: 5 };
	pub const YELLOW: PaletteColor = Self { id: 13 };
	pub const GREEN: PaletteColor = Self { id: 21 };
	pub const SLIGHTLY_LIGHT_GREEN: PaletteColor = Self { id: 29 };
	pub const LIGHT_BLUE: PaletteColor = Self { id: 37 };
	pub const BLUE: PaletteColor = Self { id: 45 };
	pub const MAGENTA: PaletteColor = Self { id: 53 };
	pub const BROWN: PaletteColor = Self { id: 61 };

	// This is not belonging to any of the columns/rows but included anyway cuz cyan is important
	pub const CYAN: PaletteColor = Self { id: 90 };
}

/// The Mk2 can light a button in different ways
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum LightMode {
	/// This is the standard mode. A straight consistent light
	Plain,
	/// A flashing motion On->Off->On->Off->...
	Flash,
	/// A smooth pulse
	Pulse,
}

/// Used for the Device Inquiry message
pub enum DeviceIdQuery {
	/// Send the Device Inquiry request to a specific device id
	Specific(u8),
	/// Send the Device Inquiry request to all devices
	Any,
}

/// Volume faders light from the bottom up, and pan faders light from the centre out.
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum FaderType {
	Volume,
	Pan,
}

/// Specifies information about a fader
pub struct Fader {
	index: u8,
	color: PaletteColor,
	initial_value: u8,
}

impl Fader {
    pub fn new(index: u8, color: PaletteColor, initial_value: u8) -> Self {
		assert!(initial_value <= 127);
		// future reader, don't attempt to raise this limit to 8, it breaks the Mk2 until you
		// reconnect :p
		assert!(index <= 7);

		return Self { index, color, initial_value };
	}

	pub fn index(&self) -> u8 { self.index }
	pub fn color(&self) -> PaletteColor { self.color }
	pub fn initial_value(&self) -> u8 { self.initial_value }
}

#[allow(dead_code)] // to prevent "variant is never constructed" warning
enum Layout {
	Session,
	User1, // drum rack
	User2,
	Reserved, // reserved for Ableton Live, shouldn't be used here
	Volume,
	Pan,
}

/// This is the handler object for the Launchpad Mk2's fader mode, in which you can utilize the
/// built-in fader functionality. You can specify for each fader its position, its color, and its
/// default value.
/// 
/// For further documentation and examples, see `LaunchpadMk2Output::enter_fader_mode()`.
pub struct FaderMode {
	output: LaunchpadMk2Output,
	fader_type: FaderType,
}

impl FaderMode {
	fn new(mut output: LaunchpadMk2Output, fader_type: FaderType) -> anyhow::Result<Self> {
		output.change_layout(match fader_type {
			FaderType::Volume => Layout::Volume,
			FaderType::Pan => Layout::Pan,
		})?;
		return Ok(Self { output, fader_type });
	}

	/// Exit fader mode by transforming this FaderMode object back into a LaunchpadMk2Output object.
	#[must_use="You must use the returned object, or the MIDI connection will be dropped"]
	pub fn exit(mut self) -> anyhow::Result<LaunchpadMk2Output> {
		self.output.change_layout(Layout::Session)?;
		return Ok(self.output);
	}

	/// Place faders on the screen. The faders' properties are specified using the `&[Fader]` slice.
	pub fn designate_faders(&mut self, faders: &[Fader]) -> anyhow::Result<()> {
		assert!(faders.len() <= 8);

		let fader_type = match self.fader_type {
			FaderType::Volume => 0,
			FaderType::Pan => 1,
		};

		let mut bytes = Vec::with_capacity(8 + 4 * faders.len());
		bytes.extend(&[240, 0, 32, 41, 2, 24, 43]);
		for fader in faders {
			bytes.extend(&[fader.index, fader_type, fader.color.id(), fader.initial_value]);
		}
		bytes.push(247);

		return self.output.send(&bytes);
	}

	/// Moves a fader, specified by `index`, to a specific `value`
	pub fn set_fader(&mut self, index: u8, value: u8) -> anyhow::Result<()> {
		assert!(index <= 7);
		assert!(value <= 127);

		return self.output.send(&[176, 21 + index, value]);
	}
}

/// The object handling any messages _to_ the launchpad. To get started, initialize with
/// `LaunchpadMk2Output::guess()` and then send messages to your liking. The connection to the
/// launchpad will get closed when this object goes out of scope.
/// 
/// For example:
/// ```
/// let mut output = LaunchpadMk2Output::guess();
/// 
/// output.light_all(PaletteColor::BLACK); // clear screen
/// 
/// // make a red cross in the center
/// output.light_row(4, PaletteColor::RED);
/// output.light_row(5, PaletteColor::RED);
/// output.light_column(4, PaletteColor::RED);
/// output.light_column(5, PaletteColor::RED);
/// 
/// // light top left button magenta
/// output.light(Button::GridButton { x: 0, y: 0 }, PaletteColor::MAGENTA);
/// ```
pub struct LaunchpadMk2Output {
	connection: MidiOutputConnection,
}

impl crate::OutputDevice for LaunchpadMk2Output {
	const MIDI_CONNECTION_NAME: &'static str = "Launchy Mk2 output";
	const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad MK2";

	fn from_connection(connection: MidiOutputConnection) -> anyhow::Result<Self> {
		let mut self_ = Self { connection };
		self_.change_layout(Layout::Session)?;
		return Ok(self_);
	}

	fn send(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
		self.connection.send(bytes)?;
		return Ok(());
	}
}

impl LaunchpadMk2Output {
	/// This is a function testing various parts of this API by executing various commands in order
	/// to find issues either in this library or in your device
	pub fn test_api(&mut self) -> anyhow::Result<()> {
		self.light_all(PaletteColor::DARK_GRAY)?;
		std::thread::sleep(std::time::Duration::from_millis(250));
		self.light_all(PaletteColor::BLACK)?;

		// Test single led lighting, only plain
		self.light(Button::ControlButton { index: 0 }, PaletteColor { id: 5 })?;
		self.light_rgb(Button::ControlButton { index: 1 }, RgbColor { r: 63, g: 0, b: 63 })?;
		self.light(Button::GridButton { x: 0, y: 0 }, PaletteColor { id: 5 })?;
		self.light_rgb(Button::GridButton { x: 1, y: 0 }, RgbColor { r: 63, g: 0, b: 63 })?;

		// Test multiple lights
		self.light_multiple(&[
			(Button::GridButton { x: 0, y: 1 }, PaletteColor { id: 18 }),
			(Button::GridButton { x: 0, y: 2 }, PaletteColor { id: 18 }),
		])?;
		self.light_multiple_rgb(&[
			(Button::GridButton { x: 0, y: 3 }, RgbColor { r: 63, g: 63, b: 63 }),
			(Button::GridButton { x: 0, y: 4 }, RgbColor { r: 63, g: 40, b: 63 }),
		])?;

		// Test pulse and flash
		self.flash(Button::GridButton { x: 1, y: 1 }, PaletteColor { id: 5 })?;
		self.pulse(Button::GridButton { x: 1, y: 2 }, PaletteColor { id: 9 })?;
		self.flash_multiple(&[
			(Button::GridButton { x: 2, y: 1 }, PaletteColor { id: 5 }),
			(Button::GridButton { x: 2, y: 2 }, PaletteColor { id: 9 }),
		])?;
		self.pulse_multiple(&[
			(Button::GridButton { x: 3, y: 1 }, PaletteColor { id: 5 }),
			(Button::GridButton { x: 3, y: 2 }, PaletteColor { id: 9 }),
		])?;
		// same but for control row
		self.flash(Button::ControlButton { index: 2 }, PaletteColor { id: 5 })?;
		self.pulse(Button::ControlButton { index: 3 }, PaletteColor { id: 9 })?;
		self.flash_multiple(&[
			(Button::ControlButton { index: 4 }, PaletteColor { id: 5 }),
			(Button::ControlButton { index: 5 }, PaletteColor { id: 9 }),
		])?;
		self.pulse_multiple(&[
			(Button::ControlButton { index: 6 }, PaletteColor { id: 5 }),
			(Button::ControlButton { index: 7 }, PaletteColor { id: 9 }),
		])?;
		
		// Test row, only grid
		self.light_rows(&[
			(7, PaletteColor { id: 16 }),
			(8, PaletteColor { id: 18 }),
			])?;
		
		// loop {
		// 	// default is 120 BPM
		// 	let bpm: f32 = 240.0; // 60-240
		// 	let interval_ms = (2500.0 / bpm).ceil() as u64;

		// 	self.send_clock_tick()?;
		// 	std::thread::sleep(std::time::Duration::from_millis(interval_ms));
		// }

		// std::thread::sleep(std::time::Duration::from_millis(1000));
		// // Test control button row
		// self.light_row(0, PaletteColor { id: 5 })?;

		return Ok(());
	}

	/// Set a `button` to a certain `color` with a certain `light_mode`.
	///
	/// For example to start a yellow pulse on the leftmost control button:
	/// ```
	/// let mut output = LaunchpadMk2Output::guess();
	/// 
	/// let button = Button::ControlButton { index: 0 };
	/// let color = PaletteColor::YELLOW;
	/// let light_mode = LightMode::Pulse;
	/// output.set_button(button, color, light_mode)?;
	/// ```
	pub fn set_button(&mut self, button: Button, color: PaletteColor, light_mode: LightMode)
			-> anyhow::Result<()> {
		
		assert!(color.id <= 127);
		
		let type_byte = match button {
			Button::GridButton { .. } => 0x90,
			Button::ControlButton { .. } => 0xB0,
		} + match light_mode {
			LightMode::Plain => 0,
			LightMode::Flash => 1,
			LightMode::Pulse => 2,
		};

		return self.send(&[type_byte, Self::encode_button(button), color.id]);
	}

	/// Like `set_button()`, but for multiple buttons. This method lights multiple buttons with
	/// varying color. The light mode can't be varied between buttons.
	///
	/// For example to start a yellow flash on the leftmost control button and a red flash on the
	/// button to the right:
	/// ```
	/// output.set_buttons(&[
	/// 	(Button::ControlButton { index: 0 }, PaletteColor::YELLOW),
	/// 	(Button::ControlButton { index: 1 }, PaletteColor::RED),
	/// ], LightMode::Flash)?;
	/// ```
	pub fn set_buttons(&mut self,
		buttons: impl IntoIterator<Item = impl std::borrow::Borrow<(Button, PaletteColor)>>,
		light_mode: LightMode
	) -> anyhow::Result<()> {
		
		let msg_type_byte = match light_mode {
			LightMode::Plain => 10,
			LightMode::Flash => 35,
			LightMode::Pulse => 40,
		};

		// I have NO IDEA why this is needed?!?! It's not in the official documentation, but
		// experimentation revealed that each packet needs to be prefixed with a dummy null byte
		// in order to work ONLY FOR FLASH AND PULSE THOUGH! why? xD
		let add_null_byte = match light_mode {
			LightMode::Plain => false,
			LightMode::Flash | LightMode::Pulse => true,
		};

		return self.send_multiple(msg_type_byte, add_null_byte, 80, buttons.into_iter()
				.map(|pair| {
					let &(button, color) = pair.borrow();
					(Self::encode_button(button), color)
				}));
	}

	/// Light multiple buttons with varying color. This method support RGB.
	/// 
	/// For example to light the top left button green and the top right button red:
	/// ```
	/// output.light_multiple_rgb(&[
	/// 	(Button::GridButton { x: 0, y: 0 }, RgbColor.new(0, 0, 63)),
	/// 	(Button::GridButton { x: 7, y: 0 }, RgbColor.new(63, 0, 0)),
	/// ])?;
	/// ```
	pub fn light_multiple_rgb<I, T>(&mut self,
		buttons: I,
	) -> anyhow::Result<()>
		where I: IntoIterator<Item = T>,
			T: std::borrow::Borrow<(Button, RgbColor)>,
			I::IntoIter: ExactSizeIterator {

		let buttons = buttons.into_iter();

		assert!(buttons.size_hint().0 <= 80);

		let mut bytes = Vec::with_capacity(8 + 12 * buttons.len());

		bytes.extend(&[240, 0, 32, 41, 2, 24, 11]);
		for pair in buttons {
			let &(button, color) = pair.borrow();
			assert!(color.is_valid());
			bytes.extend(&[Self::encode_button(button), color.r, color.g, color.b]);
		}
		bytes.push(247);

		return self.send(&bytes);
	}

	/// Light multiple columns with varying colors. This method does not light up the control
	/// buttons
	/// 
	/// For example to light the first column yellow and the second column blue:
	/// ```
	/// output.light_columns(&[
	/// 	(0, PaletteColor::YELLOW),
	/// 	(1, PaletteColor::BLUE),
	/// ])?;
	/// ```
	pub fn light_columns(&mut self,
		buttons: impl IntoIterator<Item = impl std::borrow::Borrow<(u8, PaletteColor)>>,
	) -> anyhow::Result<()> {

		return self.send_multiple(12, false, 9, buttons);
	}

	/// Light multiple row with varying colors. This method _does_ light up the side buttons.
	/// 
	/// Note: the row are counted starting at the control row! For example to light the control row
	/// magenta and the first grid row green:
	/// ```
	/// output.light_rows(&[
	/// 	(0, PaletteColor::MAGENTA),
	/// 	(1, PaletteColor::GREEN),
	/// ])?;
	/// ```
	pub fn light_rows(&mut self,
		buttons: impl IntoIterator<Item = impl std::borrow::Borrow<(u8, PaletteColor)>>,
	) -> anyhow::Result<()> {

		return self.send_multiple(13, false, 9, buttons.into_iter()
				.map(|pair| {
					let &(row, color) = pair.borrow();
					(8 - row, color)
				}));
	}

	/// Light all buttons, including control and side buttons.
	/// 
	/// For example to clear the screen:
	/// ```
	/// output.light_all(PaletteColor::BLACK)?;
	/// ```
	pub fn light_all(&mut self, color: PaletteColor) -> anyhow::Result<()> {
		return self.send(&[240, 0, 32, 41, 2, 24, 14, color.id, 247]);
	}

	/// By default, Launchpad MK2 will flash and pulse at 120 BPM. This can be altered by sending
	/// these clock ticks by calling `send_clock_tick()`. These ticks should be sent at a rate of 24
	/// per beat.
	/// 
	/// To set a tempo of 100 BPM, 2400 clock ticks should be sent each minute, or with a time
	/// interval of 25ms.
	/// 
	/// Launchpad MK2 supports tempos between 40 and 240 BPM, faster clock ticks are apparently
	/// ignored.
	/// 
	/// For example to send clock ticks at 200 BPM:
	/// ```
	/// let beats_per_minute = 200;
	/// let clock_ticks_per_second = beats_per_minute * 60 * 24;
	/// let clock_tick_interval = std::time::Duration::from_millis(1000 / clock_ticks_per_second);
	/// loop {
	/// 	output.send_clock_tick()?;
	/// 	std::thread::sleep(clock_tick_interval);
	/// }
	/// ```
	pub fn send_clock_tick(&mut self) -> anyhow::Result<()> {
		return self.send(&[248, 0, 0]);
	}

	/// Requests the Launchpad Mk2 to send a so-called device inquiry. The device inquiry contains
	/// information about the device ID and the firmware revision number.
	/// 
	/// In order to be able to receive the Launchpad Mk2's response to this request,
	/// you must have a Launchpad Mk2 input object set up.
	pub fn request_device_inquiry(&mut self, query: DeviceIdQuery) -> anyhow::Result<()> {
		const QUERY_DEVICE_ID_FOR_ANY: u8 = 127;
		
		let query_device_id = match query {
			DeviceIdQuery::Specific(device_id) => {
				assert_ne!(device_id, QUERY_DEVICE_ID_FOR_ANY);
				device_id
			},
			DeviceIdQuery::Any => QUERY_DEVICE_ID_FOR_ANY,
		};

		return self.send(&[240, 126, query_device_id, 6, 1, 247]);
	}

	/// Requests the Launchpad Mk2 to send a so-called version inquiry. The version inquiry contains
	/// information about the current bootloader and firmware versions, as well as the size of the
	/// bootloader in KB.
	/// 
	/// In order to be able to receive the Launchpad Mk2's response to this request,
	/// you must have a Launchpad Mk2 input object set up.
	pub fn request_version_inquiry(&mut self) -> anyhow::Result<()> {
		return self.send(&[240, 0, 32, 41, 0, 112, 247]);
	}

	/// Starts a text scroll across the screen. The screen is temporarily cleared. You can specify
	/// the color of the text and whether the text should loop indefinitely.
	/// 
	/// In addition to the standard ASCII characters, Launchpad MK2 recognises plain values 1 – 7 as
	/// speed commands (where 1 is the slowest and 7 is fastest). This allows the scrolling speed to
	/// be manipulated mid-text. The default speed is 4.
	/// 
	/// When the text ends, Launchpad MK2 restores the LEDs to their previous settings. As the text
	/// either ends or loops, a message will be sent to the LaunchpadMk2Input.
	///
	/// For example to scroll the text "Hello, world!" in blue; "Hello" scrolling slow and "world"
	/// fast:
	/// ```
	/// output.scroll_text(b"\x01Hello, \x07world!", PaletteColor::BLUE, false)?;
	/// ```
	pub fn scroll_text(&mut self, text: &[u8], color: PaletteColor, should_loop: bool)
			-> anyhow::Result<()> {
		
		let bytes = &[
			&[240, 0, 32, 41, 2, 24, 20, color.id(), should_loop as u8],
			text,
			&[247],
		].concat();

		return self.send(bytes);
	}

	/// Transforms this LaunchpadMk2Output object to go into "fader mode". In fader mode, you have
	/// the ability to utilize the Mk2's built-in fader functionality.
	/// 
	/// Launchpad MK2 has two virtual fader modes, one with volume style faders and one with pan
	/// style (the two styles cannot be mixed).
	/// 
	/// The fader will light up according to its current value with volume faders lighting from the
	/// bottom up, and pan faders lighting from the centre out.
	/// 
	/// When a button is pressed to change the level of a fader, Launchpad MK2 will move the fader
	/// to that position and send interim values to smooth the transition. For each interim value, a
	/// message is sent to LaunchpadMk2Input.
	/// 
	/// See [FaderMode](struct.FaderMode.html) for documentation on FaderMode's methods.
	/// 
	/// For example to place three pan faders:
	/// - A green one on the left, turned all the way down
	/// - Another green one next to the first fader, turned all the way up
	/// - A white one on the right, centered
	/// ```
	/// let mut fader_setup = output.enter_fader_mode()?;
	/// 
	/// fader_mode.designate_faders(&[
	/// 	Fader::new(0, PaletteColor::GREEN, 0),
	/// 	Fader::new(1, PaletteColor::GREEN, 127),
	/// 	Fader::new(7, PaletteColor::WHITE, 63),
	/// ])?;
	/// 
	/// let mut output = fader_setup.exit()?;
	/// ```
	#[must_use="If you don't use the returned object, the MIDI connection will be dropped immediately"]
	pub fn enter_fader_mode(self, fader_type: FaderType) -> anyhow::Result<FaderMode> {
		return FaderMode::new(self, fader_type);
	}

	/// Force the Launchpad MK2 into bootloader mode
	pub fn enter_bootloader(&mut self) -> anyhow::Result<()> {
		return self.send(&[240, 0, 32, 41, 0, 113, 0, 105, 247]);
	}

	fn change_layout(&mut self, layout: Layout) -> anyhow::Result<()> {
		let layout = match layout {
			Layout::Session => 0,
			Layout::User1 => 1,
			Layout::User2 => 2,
			Layout::Reserved => 3,
			Layout::Volume => 4,
			Layout::Pan => 5,
		};
		return self.send(&[240, 0, 32, 41, 2, 24, 34, layout, 247]);
	}

	// param `insert_null_bytes`: whether every packet should be preceeded by a null byte
	fn send_multiple(&mut self,
		msg_type_byte: u8,
		insert_null_bytes: bool,
		max_packets: usize,
		pair_iterator: impl IntoIterator<Item = impl std::borrow::Borrow<(u8, PaletteColor)>>,
	) -> anyhow::Result<()> {
		
		let pair_iterator = pair_iterator.into_iter();

		let capacity = 8 + 12 * (pair_iterator.size_hint().0 + insert_null_bytes as usize);
		let mut bytes = Vec::with_capacity(capacity);

		bytes.extend(&[240, 0, 32, 41, 2, 24, msg_type_byte]);
		for (i, pair) in pair_iterator.enumerate() {
			if i >= max_packets {
				panic!("Only {} or less elements are supported per message!", max_packets);
			}

			let &(button_specifier, color) = pair.borrow();
			if insert_null_bytes { bytes.push(0) }
			bytes.extend(&[button_specifier, color.id]);
		}
		bytes.push(247);

		return self.send(&bytes);
	}

	fn encode_button(button: Button) -> u8 {
		match button {
			Button::GridButton { x, y } => {
				assert!(x <= 8);
				assert!(y <= 7);

				return 10 * (8 - y) + x + 1;
			},
			Button::ControlButton { index } => {
				assert!(index <= 7);

				return index + 104;
			}
		}
	}

	// --------------------------------------------------------------------------------------------
	// Below this point are shorthand function
	// --------------------------------------------------------------------------------------------

	/// Light a button with a color from the Mk2 palette. Identical to
	/// `set_button(<button>, <color>, LightMode::Plain)`.
	/// 
	/// For example to light the "Volume" side button cyan:
	/// 
	/// ```
	/// output.light(Button::VOLUME, PaletteColor::CYAN)?;
	/// ```
	pub fn light(&mut self, button: Button, color: PaletteColor) -> anyhow::Result<()> {
		return self.set_button(button, color, LightMode::Plain);
	}

	/// Starts a flashing motion between the previously shown color on this button and palette color
	/// `color`, with a duty cycle of 50% and a bpm of 120. The bpm can be controlled using
	/// `send_clock_tick()`.
	/// 
	/// Identical to `set_button(<button>, <color>, LightMode::FLASH)`.
	/// 
	/// For example to start a red flash on the "Session" button at the top:
	/// ```
	/// output.flash(Button::UP, PaletteColor::RED)?;
	/// ```
	pub fn flash(&mut self, button: Button, color: PaletteColor) -> anyhow::Result<()> {
		return self.set_button(button, color, LightMode::Flash);
	}

	/// Start a pulse; a rhythmic increase and decreases in brightness. The speed can be controlled
	/// using `send_clock_tick()`. Identical to `set_button(<button>, <color>, LightMode::PULSE)`.
	/// 
	/// For example to start a magenta pulse on the top right grid button:
	/// ```
	/// output.pulse(Button::GridButton { x: 7, y: 0 }, PaletteColor::MAGENTA)?;
	/// ```
	pub fn pulse(&mut self, button: Button, color: PaletteColor) -> anyhow::Result<()> {
		return self.set_button(button, color, LightMode::Pulse);
	}

	/// Light a single column, specified by `column` (0-8).
	/// 
	/// For example to light the entire side button column white:
	/// ```
	/// output.light_column(8, PaletteColor::WHITE)?;
	/// ```
	pub fn light_column(&mut self, column: u8, color: PaletteColor)
			-> anyhow::Result<()> {
		
		return self.light_columns(&[(column, color)]);
	}

	/// Light a single row, specified by `row` (0-8). Note: the row counting begins at the control
	/// row! So e.g. when you want to light the first grid row, pass `1` not `0`.
	/// 
	/// For example to light the first grid row green:
	/// ```
	/// output.light_row(1, PaletteColor::GREEN)?;
	/// ```
	pub fn light_row(&mut self, row: u8, color: PaletteColor)
			-> anyhow::Result<()> {
		
		return self.light_rows(&[(row, color)]);
	}

	/// Light a single button with an RGB color.
	/// 
	/// For example to light the bottom right button cyan:
	/// ```
	/// output.light_rgb(Button::GridButton { x: 7, y: 7 }, RgbColor::new(0, 63, 63))?;
	/// ```
	pub fn light_rgb(&mut self, button: Button, color: RgbColor) -> anyhow::Result<()> {
		return self.light_multiple_rgb(&[(button, color)]);
	}

	/// Light multiple buttons with varying colors. Identical to
	/// `set_buttons(<pairs>, LightMode::Plain)`
	/// 
	/// For example to light both User 1 and User 2 buttons orange:
	/// ```
	/// output.light_multiple(&[
	/// 	(Button::USER_1, PaletteColor::new(9)),
	/// 	(Button::USER_2, PaletteColor::new(9)),
	/// ])?;
	/// ```
	pub fn light_multiple(&mut self,
		buttons: impl IntoIterator<Item = impl std::borrow::Borrow<(Button, PaletteColor)>>,
	) -> anyhow::Result<()> {

		return self.set_buttons(buttons, LightMode::Plain);
	}

	/// Start flashing multiple buttons with varying colors. Identical to
	/// `set_buttons(<pairs>, LightMode::Flash)`
	/// 
	/// For example to flash both User 1 and User 2 buttons orange:
	/// ```
	/// output.flash_multiple(&[
	/// 	(Button::USER_1, PaletteColor::new(9)),
	/// 	(Button::USER_2, PaletteColor::new(9)),
	/// ])?;
	/// ```
	pub fn flash_multiple(&mut self,
		buttons: impl IntoIterator<Item = impl std::borrow::Borrow<(Button, PaletteColor)>>,
	) -> anyhow::Result<()> {

		return self.set_buttons(buttons, LightMode::Flash);
	}

	/// Start pulsing multiple buttons with varying colors. Identical to
	/// `set_buttons(<pairs>, LightMode::Pulse)`
	/// 
	/// For example to pulse both User 1 and User 2 buttons orange:
	/// ```
	/// output.pulse_multiple(&[
	/// 	(Button::USER_1, PaletteColor::new(9)),
	/// 	(Button::USER_2, PaletteColor::new(9)),
	/// ])?;
	/// ```
	pub fn pulse_multiple(&mut self,
		buttons: impl IntoIterator<Item = impl std::borrow::Borrow<(Button, PaletteColor)>>,
	) -> anyhow::Result<()> {
		
		return self.set_buttons(buttons, LightMode::Pulse);
	}

	/// Clears the entire field of buttons. Equivalent to `output.light_all(PaletteColor::BLACK)`.
	pub fn clear(&mut self) -> anyhow::Result<()> {
		return self.light_all(PaletteColor::BLACK);
	}
}