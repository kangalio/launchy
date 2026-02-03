use midir::MidiOutputConnection;

pub use crate::protocols::query::*;

use super::Button;
use crate::OutputDevice;

/// The maximum value of an RGB LED
const MAX_RGB: u8 = 127;

/// A color from the Mk3 color palette. See the "Launchpad MK3 Programmers Reference Manual"
/// to see the palette, or [see here](http://launchpaddr.com/mk3palette/).
///
/// Everywhere where a PaletteColor is expected as a funcion argument, you can also directly pass
/// in the palette index and call `.into()` on it. Example:
/// ```no_run
/// # use launchy::mini_mk3::{PaletteColor};
/// # let output: launchy::mini_mk3::Output = unimplemented!();
/// // This:
/// output.light_all(PaletteColor::new(92));
/// // can also be written as:
/// output.light_all(92.into());
/// # Ok::<(), launchy::MidiError>(())
/// ```
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct PaletteColor {
    pub(crate) id: u8,
}

impl PaletteColor {
    pub fn is_valid(&self) -> bool {
        self.id <= 127
    }

    pub fn new(id: u8) -> Self {
        let self_ = Self { id };
        assert!(self_.is_valid());
        self_
    }

    pub fn id(&self) -> u8 {
        self.id
    }
    pub fn set_id(&mut self, id: u8) {
        self.id = id
    }
}

impl From<u8> for PaletteColor {
    fn from(id: u8) -> Self {
        Self::new(id)
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
/// An RGB color. Each component may only go up to 63.
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
        self_
    }

    /// Check whether the rgb color is valid - each component may only go up to MAX_RGB.
    pub fn is_valid(&self) -> bool {
        self.r <= MAX_RGB && self.g <= MAX_RGB && self.b <= MAX_RGB
    }

    pub fn red(&self) -> u8 {
        self.r
    }
    pub fn green(&self) -> u8 {
        self.g
    }
    pub fn blue(&self) -> u8 {
        self.b
    }
    pub fn set_red(&mut self, r: u8) {
        assert!(r <= MAX_RGB);
        self.r = r
    }
    pub fn set_green(&mut self, g: u8) {
        assert!(g <= MAX_RGB);
        self.g = g
    }
    pub fn set_blue(&mut self, b: u8) {
        assert!(b <= MAX_RGB);
        self.b = b
    }
}

/// The button styles supported by the MK3 Mini
///
/// Buttons can be in one of 3 states:
///
/// - Plain: a constant color, using either a palette color or an RGB color.
/// - Flashing: flashing between two colors on a 50% duty cycle. For simplicity,
///   [ButtonStyle::flash] flashes between a given color and black, and [ButtonStyle::flash2]
///   flashes between 2 colors. Flashing can only use palette colors.
/// - Pulsing: pulsing between a given (palette) color and black. Pulsing looks more subdued than
///   flashing.
///
/// [PaletteColor] and [RgbColor] are convertible into [ButtonStyle]
/// using `color.into()`; this will use the plain (non-flashing) button
/// style.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum ButtonStyle {
    Palette {
        color: PaletteColor,
    },
    Rgb {
        color: RgbColor,
    },
    Flash {
        color1: PaletteColor,
        color2: PaletteColor,
    },
    Pulse {
        color: PaletteColor,
    },
}

impl ButtonStyle {
    /// Create a plain button style from a palette color
    pub fn palette(color: PaletteColor) -> Self {
        ButtonStyle::Palette { color }
    }

    /// Flash between a given color and black
    pub fn flash(color: PaletteColor) -> Self {
        Self::flash2(color, PaletteColor::BLACK)
    }

    /// Flash between a given color and black
    pub fn flash2(color1: PaletteColor, color2: PaletteColor) -> Self {
        ButtonStyle::Flash { color1, color2 }
    }

    /// Pulse the given color (and black)
    pub fn pulse(color: PaletteColor) -> Self {
        ButtonStyle::Pulse { color }
    }

    /// Create a plain button style from an RGB color
    pub fn rgb(color: RgbColor) -> Self {
        ButtonStyle::Rgb { color }
    }

    /// Validate that the button style's colors only use valid numbers
    pub fn is_valid(&self) -> bool {
        match self {
            ButtonStyle::Palette { color } => color.is_valid(),
            ButtonStyle::Rgb { color } => color.is_valid(),
            ButtonStyle::Flash { color1, color2 } => color1.is_valid() && color2.is_valid(),
            ButtonStyle::Pulse { color } => color.is_valid(),
        }
    }
}

impl From<PaletteColor> for ButtonStyle {
    fn from(color: PaletteColor) -> Self {
        ButtonStyle::Palette { color }
    }
}

impl From<&PaletteColor> for ButtonStyle {
    fn from(color: &PaletteColor) -> Self {
        Self::from(*color)
    }
}

impl From<RgbColor> for ButtonStyle {
    fn from(color: RgbColor) -> Self {
        ButtonStyle::Rgb { color }
    }
}

impl From<&RgbColor> for ButtonStyle {
    fn from(color: &RgbColor) -> Self {
        Self::from(*color)
    }
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
    pub const LIGHT_RED: PaletteColor = Self { id: 4 };
    pub const RED: PaletteColor = Self { id: 5 };
    pub const ORANGE: PaletteColor = Self { id: 9 };
    pub const YELLOW: PaletteColor = Self { id: 13 };
    pub const LIME_GREEN: PaletteColor = Self { id: 17 };
    pub const GREEN: PaletteColor = Self { id: 21 };
    pub const SLIGHTLY_LIGHT_GREEN: PaletteColor = Self { id: 29 };
    pub const LIGHT_BLUE: PaletteColor = Self { id: 37 };
    pub const BLUE: PaletteColor = Self { id: 45 };
    pub const PURPLE: PaletteColor = Self { id: 49 };
    pub const MAGENTA: PaletteColor = Self { id: 53 };
    pub const PINK: PaletteColor = Self { id: 57 };
    pub const BROWN: PaletteColor = Self { id: 61 };

    // This is not belonging to any of the columns/rows but included anyway cuz cyan is important
    pub const CYAN: PaletteColor = Self { id: 90 };
}

/// The Mini Mk3 can light a button in different ways
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum LightMode {
    /// This is the standard mode. A straight consistent light
    Plain,
    /// A flashing motion On->Off->On->Off->...
    Flash,
    /// A smooth pulse
    Pulse,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum SleepMode {
    Sleep = 0,
    Wake = 1,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Layout {
    Live = 0, // reserved for Ableton Live, shouldn't be used here
    Programmer = 1,
}

impl From<u8> for Layout {
    fn from(id: u8) -> Self {
        match id {
            0 => Self::Live,
            1 => Self::Programmer,
            _ => panic!("Unexpected layout id {}", id),
        }
    }
}

/// The object handling any messages _to_ the launchpad. To get started, initialize with
/// [(`Output::guess`)[OutputDevice::guess]] and then send messages to your liking. The connection
/// to the launchpad will get closed when this object goes out of scope.
///
/// For example:
/// ```no_run
/// # use launchy::OutputDevice as _;
/// # use launchy::mini_mk3::{PaletteColor, Button, Output};
/// let mut output = Output::guess()?;
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
/// # Ok::<(), launchy::MidiError>(())
/// ```
///
/// # Representing color
/// The Launchpad Mk3 has two different ways to represent color. You can either use one of the 128
/// built-in palette colors, or you can create a custom color with custom rgb components.
/// Why would you choose the palette colors when you can just create your required colors yourself?
/// Well some operations on the Mk3 only support palette colors. Besides, sending palette color midi
/// messages is simply faster. Therefore you should aim to use the palette colors when possible.
pub struct Output {
    connection: MidiOutputConnection,
}

impl crate::OutputDevice for Output {
    const MIDI_CONNECTION_NAME: &'static str = "Launchy Mini Mk3 output";

    /// Device name.
    ///
    /// On MacOS, the Mini MK3 advertises:
    ///
    /// - "Launchpad Mini MK3 LPMiniMK3 DAW"
    /// - "Launchpad Mini MK3 LPMiniMK3 MIDI"
    ///
    /// But only the MIDI interface works for what we want to do, so include the "MIDI" string.
    const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad Mini MK3 LPMiniMK3 MIDI";

    fn from_connection(connection: MidiOutputConnection) -> Result<Self, crate::MidiError> {
        let mut self_ = Self { connection };
        self_.change_layout(Layout::Programmer)?;
        Ok(self_)
    }

    fn send(&mut self, bytes: &[u8]) -> Result<(), crate::MidiError> {
        self.connection.send(bytes)?;
        Ok(())
    }
}

impl Output {
    /// Set a `button` to a certain `color` with a certain `light_mode`.
    ///
    /// This uses a direct MIDI message on channel 1, 2 or 3 to set the color,
    /// and uses different data structures than `set_buttons` (which uses a
    /// SysEx message that also allows RGB button colors and flashing between 2
    /// different palette colors).
    ///
    /// For example to start a yellow pulse on the leftmost control button:
    /// ```no_run
    /// # use launchy::mini_mk3::{PaletteColor, Button, LightMode};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// let button = Button::ControlButton { index: 0 };
    /// let color = PaletteColor::YELLOW;
    /// let light_mode = LightMode::Pulse;
    /// output.set_button(button, color, light_mode)?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn set_button(
        &mut self,
        button: Button,
        color: PaletteColor,
        light_mode: LightMode,
    ) -> Result<(), crate::MidiError> {
        assert!(color.id <= 127);

        let type_byte = match button {
            Button::GridButton { .. } => 0x90,
            Button::ControlButton { .. } => 0xB0,
        } + match light_mode {
            LightMode::Plain => 0,
            LightMode::Flash => 1,
            LightMode::Pulse => 2,
        };

        self.send(&[type_byte, Self::encode_button(button), color.id])
    }

    /// Light multiple buttons with varying colors.
    ///
    /// This uses a SysEx message to set one or more buttons in one go, and supports
    /// RGB colors as well as flashing between 2 colors.
    ///
    /// Apart from `set_button` which uses a simpler mechanism to set button colors,
    /// all other lighting control methods forward to this one.
    ///
    /// For example to light the top left button green and the top right button red:
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, ButtonStyle, RgbColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.set_buttons(&[
    ///     (Button::GridButton { x: 0, y: 0 }, ButtonStyle::rgb(RgbColor::new(0, 0, 127))),
    ///     (Button::GridButton { x: 7, y: 0 }, RgbColor::new(127, 0, 0).into()),
    /// ])?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn set_buttons<I, T>(&mut self, buttons: I) -> Result<(), crate::MidiError>
    where
        I: IntoIterator<Item = T>,
        T: std::borrow::Borrow<(Button, ButtonStyle)>,
    {
        let buttons = buttons.into_iter();

        assert!(buttons.size_hint().0 <= 81);

        let mut bytes = Vec::with_capacity(8 + 5 * buttons.size_hint().1.unwrap_or(40));

        // SysEx message
        bytes.extend(&[240, 0, 32, 41, 2, 13, 3]);
        for pair in buttons {
            let (button, style) = pair.borrow();
            assert!(style.is_valid());
            match style {
                ButtonStyle::Palette { color } => {
                    bytes.extend([0, Self::encode_button(*button), color.id()])
                }
                ButtonStyle::Rgb { color } => bytes.extend([
                    3,
                    Self::encode_button(*button),
                    color.red(),
                    color.green(),
                    color.blue(),
                ]),
                ButtonStyle::Flash { color1, color2 } => {
                    // Order color2 and color1 such that (1=red, 2=black) looks exactly
                    // the same as a legacy flash red (as configured by `set_button`).
                    bytes.extend([1, Self::encode_button(*button), color2.id(), color1.id()])
                }
                ButtonStyle::Pulse { color } => {
                    bytes.extend([2, Self::encode_button(*button), color.id()])
                }
            }
        }
        bytes.push(247);

        self.send(&bytes)
    }

    /// Light multiple buttons with varying RGB colors.
    ///
    /// For example to light the top left button green and the top right button red:
    ///
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, RgbColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.light_multiple_rgb(&[
    ///     (Button::GridButton { x: 0, y: 0 }, RgbColor::new(0, 0, 127)),
    ///     (Button::GridButton { x: 7, y: 0 }, RgbColor::new(127, 0, 0)),
    /// ])?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    ///
    /// The implementation of this method forwards the call to [Output::set_buttons].
    pub fn light_multiple_rgb<I, T>(&mut self, buttons: I) -> Result<(), crate::MidiError>
    where
        I: IntoIterator<Item = T>,
        T: std::borrow::Borrow<(Button, RgbColor)>,
        I::IntoIter: ExactSizeIterator,
    {
        self.set_buttons(
            buttons
                .into_iter()
                .map(|pair| *pair.borrow())
                .map(|(button, color)| (button, color.into())),
        )
    }

    /// Light multiple columns with varying colors. This method does not light up the control
    /// buttons
    ///
    /// For example to light the first column yellow and the second column blue:
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, PaletteColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.light_columns(&[
    ///     (0, PaletteColor::YELLOW),
    ///     (1, PaletteColor::BLUE),
    /// ])?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn light_columns(
        &mut self,
        buttons: impl IntoIterator<Item = impl std::borrow::Borrow<(u8, PaletteColor)>>,
    ) -> Result<(), crate::MidiError> {
        self.set_buttons(
            buttons
                .into_iter()
                .map(|pair| *pair.borrow())
                .flat_map(|(col, color)| {
                    column_buttons(col).map(move |button| (button, color.into()))
                }),
        )
    }

    /// Light multiple row with varying colors. This method _does_ light up the side buttons.
    ///
    /// Note: the row are counted starting at the control row! For example to light the control row
    /// magenta and the first grid row green:
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, PaletteColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.light_rows(&[
    ///     (0, PaletteColor::MAGENTA),
    ///     (1, PaletteColor::GREEN),
    /// ])?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn light_rows(
        &mut self,
        buttons: impl IntoIterator<Item = impl std::borrow::Borrow<(u8, PaletteColor)>>,
    ) -> Result<(), crate::MidiError> {
        self.set_buttons(
            buttons
                .into_iter()
                .map(|pair| *pair.borrow())
                .flat_map(|(row, color)| {
                    row_buttons(row).map(move |button| (button, color.into()))
                }),
        )
    }

    /// Light all buttons, including control and side buttons.
    ///
    /// For example to clear the screen:
    /// ```no_run
    /// # use launchy::mini_mk3::PaletteColor;
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.light_all(PaletteColor::BLACK)?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn light_all(&mut self, color: PaletteColor) -> Result<(), crate::MidiError> {
        let mut buffer = vec![240, 0, 32, 41, 2, 13, 3];

        for row in 1..10 {
            for column in 1..10 {
                buffer.push(0);

                buffer.push(row * 10 + column);

                buffer.push(color.id);
            }
        }

        buffer.push(247);

        self.send(&buffer)
    }

    /// By default, Launchpad MK3 will flash and pulse at 120 BPM. This can be altered by sending
    /// these clock ticks by calling `send_clock_tick()`. These ticks should be sent at a rate of 24
    /// per beat.
    ///
    /// To set a tempo of 100 BPM, 2400 clock ticks should be sent each minute, or with a time
    /// interval of 25ms.
    ///
    /// Launchpad MK3 supports tempos between 40 and 240 BPM, faster clock ticks are apparently
    /// ignored.
    ///
    /// For example to send clock ticks at 200 BPM:
    /// ```no_run
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// let beats_per_minute = 200;
    /// let clock_ticks_per_second = beats_per_minute * 60 * 24;
    /// let clock_tick_interval = std::time::Duration::from_millis(1000 / clock_ticks_per_second);
    /// loop {
    ///     output.send_clock_tick()?;
    ///     std::thread::sleep(clock_tick_interval);
    /// }
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn send_clock_tick(&mut self) -> Result<(), crate::MidiError> {
        self.send(&[248, 0, 0])
    }

    /// Requests the Launchpad Mk3 to send a so-called device inquiry. The device inquiry contains
    /// information about the device ID and the firmware revision number.
    ///
    /// According to the documentation, this should return both a
    /// [super::Message::ApplicationVersion] as well as a [super::Message::BootloaderVersion].
    ///
    /// In order to be able to receive the Launchpad Mk3's response to this
    /// request, you must have a Launchpad Mk3 input object set up.
    pub fn request_device_inquiry(&mut self, query: DeviceIdQuery) -> Result<(), crate::MidiError> {
        request_device_inquiry(self, query)
    }

    /// Requests the Launchpad Mk3 to send a `Message::SleepState` message.
    ///
    /// In order to be able to receive the Launchpad Mk3's response to this request,
    /// you must have a Launchpad Mk3 input object set up.
    pub fn request_sleep_mode(&mut self) -> Result<(), crate::MidiError> {
        self.send(&[240, 0, 32, 41, 2, 13, 9, 247])
    }

    /// Starts a text scroll across the screen.
    ///
    /// The screen is temporarily cleared. You can specify the color of the text
    /// and whether the text should loop indefinitely.
    ///
    /// Speed is given in pads/second, and must be in range (0..128). 16 is
    /// already quite a brisk speed. If speed is >= 64, it is interpreted as a
    /// negative number, formed by subtracing 128 from it. This will make the
    /// text scroll from left to right.
    ///
    /// If text is the empty string, the attributes of the currently ongoing
    /// scroll will be changed instead (color, speed, looping).
    ///
    /// When the text ends, Launchpad MK3 restores the LEDs to their previous
    /// settings.
    ///
    /// WARNING: the Mini MK3 does not seem to have a Message that tells you
    /// when the text is finished. As such, you will have to do time
    /// calculations yourself to know when your text has finished scrolling.
    /// If your text loops, you must call [Output::stop_scroll] at some point.
    ///
    /// For example to scroll the text "Hello, world!" in blue:
    ///
    /// ```no_run
    /// # use launchy::mini_mk3::PaletteColor;
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.scroll_text(b"Hello, world!", PaletteColor::BLUE, 32, false)?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn scroll_text(
        &mut self,
        text: &[u8],
        color: PaletteColor,
        speed: u8,
        should_loop: bool,
    ) -> Result<(), crate::MidiError> {
        assert!((0..128).contains(&speed));
        let bytes = &[
            &[
                240,
                0,
                32,
                41,
                2,
                13,
                7,
                should_loop as u8,
                speed,
                // Palette color
                0,
                color.id(),
            ],
            text,
            &[247],
        ]
        .concat();

        self.send(bytes)
    }

    /// Stop the ongoing text scroll immediately
    pub fn stop_scroll(&mut self) -> Result<(), crate::MidiError> {
        self.send(&[240, 0, 32, 41, 2, 13, 7 /* No text */, 247])
    }

    pub fn send_sleep(&mut self, sleep_mode: SleepMode) -> Result<(), crate::MidiError> {
        self.send(&[240, 0, 32, 41, 2, 13, 9, sleep_mode as u8, 247])
    }

    // /// Force the Launchpad MK3 into bootloader mode
    // pub fn enter_bootloader(&mut self) -> Result<(), crate::MidiError> {
    //     self.send(&[240, 0, 32, 41, 0, 113, 0, 105, 247])
    // }

    /// Set Board mode
    /// This is required for the mini mk3 to function properly
    ///
    /// For example to swap the board to programmer mode:
    /// ```no_run
    /// # use launchy::mini_mk3::{Layout};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// let layout_mode = Layout::Programmer;
    /// output.change_layout(layout_mode)?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn change_layout(&mut self, layout: Layout) -> Result<(), crate::MidiError> {
        self.send(&[240, 0, 32, 41, 2, 13, 14, layout as u8, 247])
    }

    fn encode_button(button: Button) -> u8 {
        match button {
            Button::GridButton { x, y } => {
                assert!(x <= 8);
                assert!(y <= 7);

                10 * (8 - y) + x + 1
            }
            Button::ControlButton { index } => {
                assert!(index <= 15);

                if index <= 7 {
                    index + 91
                } else {
                    (8 - (index - 8)) * 10 + 9
                }
            }
        }
    }

    // --------------------------------------------------------------------------------------------
    // Below this point are shorthand function
    // --------------------------------------------------------------------------------------------

    /// Put the Launchpad MK3 to sleep
    pub fn sleep(&mut self) -> Result<(), crate::MidiError> {
        self.send_sleep(SleepMode::Sleep)
    }

    /// Wake the device up from sleep mode
    pub fn wake(&mut self) -> Result<(), crate::MidiError> {
        self.send_sleep(SleepMode::Wake)
    }

    /// Light a button with a color from the Mk3 palette. Identical to
    /// `set_button(<button>, <color>, LightMode::Plain)`.
    ///
    /// For example to light the "Volume" side button cyan:
    ///
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, PaletteColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.light(Button::VOLUME, PaletteColor::CYAN)?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn light(&mut self, button: Button, color: PaletteColor) -> Result<(), crate::MidiError> {
        self.set_button(button, color, LightMode::Plain)
    }

    /// Starts a flashing motion between the previously shown color on this button and palette color
    /// `color`, with a duty cycle of 50% and a bpm of 120. The bpm can be controlled using
    /// `send_clock_tick()`.
    ///
    /// Identical to `set_button(<button>, <color>, LightMode::FLASH)`.
    ///
    /// For example to start a red flash on the "Session" button at the top:
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, PaletteColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.flash(Button::UP, PaletteColor::RED)?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn flash(&mut self, button: Button, color: PaletteColor) -> Result<(), crate::MidiError> {
        self.set_button(button, color, LightMode::Flash)
    }

    /// Start a pulse; a rhythmic increase and decreases in brightness. The speed can be controlled
    /// using `send_clock_tick()`. Identical to `set_button(<button>, <color>, LightMode::PULSE)`.
    ///
    /// For example to start a magenta pulse on the top right grid button:
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, PaletteColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.pulse(Button::GridButton { x: 7, y: 0 }, PaletteColor::MAGENTA)?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn pulse(&mut self, button: Button, color: PaletteColor) -> Result<(), crate::MidiError> {
        self.set_button(button, color, LightMode::Pulse)
    }

    /// Light a single column, specified by `column` (0-8).
    ///
    /// For example to light the entire side button column white:
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, PaletteColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.light_column(8, PaletteColor::WHITE)?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn light_column(
        &mut self,
        column: u8,
        color: PaletteColor,
    ) -> Result<(), crate::MidiError> {
        self.light_columns([(column, color)])
    }

    /// Light a single row, specified by `row` (0-8). Note: the row counting begins at the control
    /// row! So e.g. when you want to light the first grid row, pass `1` not `0`.
    ///
    /// For example to light the first grid row green:
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, PaletteColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.light_row(1, PaletteColor::GREEN)?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn light_row(&mut self, row: u8, color: PaletteColor) -> Result<(), crate::MidiError> {
        self.light_rows([(row, color)])
    }

    /// Light a single button with an RGB color.
    ///
    /// For example to light the bottom right button cyan:
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, RgbColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.light_rgb(Button::GridButton { x: 7, y: 7 }, RgbColor::new(0, 127, 127))?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn light_rgb(&mut self, button: Button, color: RgbColor) -> Result<(), crate::MidiError> {
        self.light_multiple_rgb([(button, color)])
    }

    /// Light multiple buttons with varying colors. Identical to
    /// `set_buttons(<pairs>, LightMode::Plain)`
    ///
    /// For example to light both User 1 and User 2 buttons orange:
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, PaletteColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.light_multiple(&[
    ///     (Button::USER_1, PaletteColor::new(9)),
    ///     (Button::USER_2, PaletteColor::new(9)),
    /// ])?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn light_multiple(
        &mut self,
        buttons: impl IntoIterator<Item = impl std::borrow::Borrow<(Button, PaletteColor)>>,
    ) -> Result<(), crate::MidiError> {
        self.set_buttons(
            buttons
                .into_iter()
                .map(|pair| *pair.borrow())
                .map(|(button, color)| (button, color.into())),
        )
    }

    /// Start flashing multiple buttons with varying colors. Identical to
    /// `set_buttons(<pairs>, LightMode::Flash)`
    ///
    /// For example to flash both User 1 and User 2 buttons orange:
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, PaletteColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.flash_multiple(&[
    ///     (Button::USER_1, PaletteColor::new(9)),
    ///     (Button::USER_2, PaletteColor::new(9)),
    /// ])?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn flash_multiple(
        &mut self,
        buttons: impl IntoIterator<Item = impl std::borrow::Borrow<(Button, PaletteColor)>>,
    ) -> Result<(), crate::MidiError> {
        self.set_buttons(
            buttons
                .into_iter()
                .map(|pair| *pair.borrow())
                .map(|(button, color)| (button, ButtonStyle::flash(color))),
        )
    }

    /// Start pulsing multiple buttons with varying colors. Identical to
    /// `set_buttons(<pairs>, LightMode::Pulse)`
    ///
    /// For example to pulse both User 1 and User 2 buttons orange:
    /// ```no_run
    /// # use launchy::mini_mk3::{Button, PaletteColor};
    /// # let output: launchy::mini_mk3::Output = unimplemented!();
    /// output.pulse_multiple(&[
    ///     (Button::USER_1, PaletteColor::new(9)),
    ///     (Button::USER_2, PaletteColor::new(9)),
    /// ])?;
    /// # Ok::<(), launchy::MidiError>(())
    /// ```
    pub fn pulse_multiple(
        &mut self,
        buttons: impl IntoIterator<Item = impl std::borrow::Borrow<(Button, PaletteColor)>>,
    ) -> Result<(), crate::MidiError> {
        self.set_buttons(
            buttons
                .into_iter()
                .map(|pair| *pair.borrow())
                .map(|(button, color)| (button, ButtonStyle::pulse(color))),
        )
    }

    /// Set the LED brightness on a scale of (0..128) (exclusive).
    pub fn set_brightness(&mut self, brightness: u8) -> Result<(), crate::MidiError> {
        assert!((0..128).contains(&brightness));
        self.send(&[240, 0, 32, 41, 2, 13, 8, brightness, 247])
    }

    /// Requests a [super::Message::Brightness]
    pub fn request_brightness(&mut self) -> Result<(), crate::MidiError> {
        self.send(&[240, 0, 32, 41, 2, 13, 8, 247])
    }

    /// Clears the entire field of buttons. Equivalent to `output.light_all(PaletteColor::BLACK)`.
    pub fn clear(&mut self) -> Result<(), crate::MidiError> {
        self.light_all(PaletteColor::BLACK)
    }
}

/// Returns an iterator for all buttons in the given column
fn column_buttons(x: u8) -> impl Iterator<Item = Button> {
    (0..8).map(move |y| Button::GridButton { x, y })
}

/// Returns an iterator for all buttons in the given row
///
/// Includes the arrow buttons on the right-hand side
fn row_buttons(y: u8) -> impl Iterator<Item = Button> {
    (0..8)
        .map(move |x| Button::GridButton { x, y })
        .chain([Button::ControlButton { index: 8 + y }])
}
