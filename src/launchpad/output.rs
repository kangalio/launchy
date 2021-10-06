use midir::MidiOutputConnection;

pub use crate::protocols::query::*;

use super::Button;
use crate::OutputDevice;

/// A color from the Mk2 color palette. See the "Launchpad MK2 Programmers Reference Manual"
/// to see the palette, or [see here](http://launchpaddr.com/mk2palette/).
///
/// Everywhere where a PaletteColor is expected as a funcion argument, you can also directly pass
/// in the palette index and call `.into()` on it. Example:
/// ```no_run
/// # use launchy::mk2::{PaletteColor};
/// # let output: launchy::mk2::Output = unimplemented!();
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
        self_
    }

    /// Check whether the rgb color is valid - each component may only go up to 63.
    pub fn is_valid(&self) -> bool {
        self.r <= 63 && self.g <= 63 && self.b <= 63
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
        assert!(r <= 63);
        self.r = r
    }
    pub fn set_green(&mut self, g: u8) {
        assert!(g <= 63);
        self.g = g
    }
    pub fn set_blue(&mut self, b: u8) {
        assert!(b <= 63);
        self.b = b
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


#[allow(dead_code)] // to prevent "variant is never constructed" warning
enum Layout {
    Session,
    User1, // drum rack
    User2,
    Reserved, // reserved for Ableton Live, shouldn't be used here
    Volume,
    Pan,
}


/// The object handling any messages _to_ the launchpad. To get started, initialize with
/// [(`Output::guess`)[OutputDevice::guess]] and then send messages to your liking. The connection
/// to the launchpad will get closed when this object goes out of scope.
///
/// For example:
/// ```no_run
/// # use launchy::OutputDevice as _;
/// # use launchy::mk2::{PaletteColor, Button, Output};
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
/// The Launchpad Mk2 has two different ways to represent color. You can either use one of the 128
/// built-in palette colors, or you can create a custom color with custom rgb components.
/// Why would you choose the palette colors when you can just create your required colors yourself?
/// Well some operations on the Mk2 only support palette colors. Besides, sending palette color midi
/// messages is simply faster. Therefore you should aim to use the palette colors when possible.
pub struct Output {
    connection: MidiOutputConnection,
}

impl crate::OutputDevice for Output {
    const MIDI_CONNECTION_NAME: &'static str = "Launchy Launchpad output";
    const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad MIDI";

    fn from_connection(connection: MidiOutputConnection) -> Result<Self, crate::MidiError> {
        let mut self_ = Self { connection };
        self_.change_layout(Layout::Session)?;
        Ok(self_)
    }


    fn send(&mut self, bytes: &[u8]) -> Result<(), crate::MidiError> {
        self.connection.send(bytes)?;
        Ok(())
    }
}

impl Output {
    /// This is a function testing various parts of this API by executing various commands in order
    /// to find issues either in this library or in your device
    pub fn test_api(&mut self) -> Result<(), crate::MidiError> {
        self.light_all(PaletteColor::DARK_GRAY)?;

        Ok(())
    }

    fn change_layout(&mut self, layout: Layout) -> Result<(), crate::MidiError> {
        let layout = match layout {
            Layout::Session => 0,
            Layout::User1 => 1,
            Layout::User2 => 2,
            Layout::Reserved => 3,
            Layout::Volume => 4,
            Layout::Pan => 5,
        };
        self.send(&[240, 0, 32, 41, 2, 24, 34, layout, 247])
    }

    pub fn set_button(
        &mut self,
        button: Button,
        color: PaletteColor,
        light_mode: LightMode,
    ) -> Result<(), crate::MidiError> {
        assert!(color.id <= 127);

        let type_byte = match button {
            Button::GridButton { .. } => 144,
            Button::ControlButton { .. } => 176,
        };

        self.send(&[type_byte, Self::encode_button(button), color.id])
    }

    fn encode_button(button: Button) -> u8 {
        match button {
            Button::GridButton { x, y } => {
                assert!(x <= 8);
                assert!(y <= 7);

                y * 16 + x
            }
            Button::ControlButton { index } => {
                assert!(index <= 7);

                index + 104
            }
        }
    }

    pub fn light_multiple_rgb<I, T>(&mut self, buttons: I) -> Result<(), crate::MidiError>
        where
            I: IntoIterator<Item = T>,
            T: std::borrow::Borrow<(Button, RgbColor)>,
            I::IntoIter: ExactSizeIterator,
    {
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

        self.send(&bytes)
    }

    pub fn light_all(&mut self, color : PaletteColor) -> Result<(), crate::MidiError> {
        for x in 0..9 {
            for y in 0..8 {
                self.set_button(Button::GridButton {x, y}, color, LightMode::Plain)?;
            }
        }

        for x in 0..8 {
            self.set_button(Button::ControlButton {index : x}, color, LightMode::Plain)?;
        }

        Ok(())
    }

    /// Clears the entire field of buttons. Equivalent to `output.light_all(PaletteColor::BLACK)`.
    pub fn clear(&mut self) -> Result<(), crate::MidiError> {
        self.light_all(PaletteColor::BLACK)
    }
}
