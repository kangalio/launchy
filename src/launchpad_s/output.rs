use midir::MidiOutputConnection;

use super::Button;
use crate::OutputDevice;

pub use crate::protocols::double_buffering::*;
pub use crate::protocols::query::*;

/**
The Launchpad S output connection handler.

# Double buffering
To make more economical use of data, the Launchpad has a feature called double buffering.
Essentially, Launchpad manages two sets of LED data - buffers - for each pad. By default, these
are configured so that the buffer that is updated by incoming MIDI messages is the same as the
one that is visible, so that note-on messages immediately change their respective pads. However,
the buffers can also be configured so that Launchpad’s LED status is updated invisibly. With a
single command, these buffers can then be swapped. The pads will instantly update to show their
pre-programmed state, while the pads can again be updated invisibly ready for the next swap. The
visible buffer can alternatively be configured to swap automatically at 280ms intervals in order
to configure LEDs to flash.
*/
pub struct Output {
    connection: MidiOutputConnection,
}

impl crate::OutputDevice for Output {
    const MIDI_CONNECTION_NAME: &'static str = "Launchy S output";
    const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad S";

    fn from_connection(connection: MidiOutputConnection) -> Result<Self, crate::MidiError> {
        Ok(Self { connection })
    }

    fn send(&mut self, bytes: &[u8]) -> Result<(), crate::MidiError> {
        self.connection.send(bytes)?;
        Ok(())
    }
}

impl Output {
    /// Updates the state for a single LED, specified by `button`. The color, as well as the double
    /// buffering attributes, are specified in `light_state`.
    pub fn set_button(
        &mut self,
        button: Button,
        color: Color,
        d: DoubleBufferingBehavior,
    ) -> Result<(), crate::MidiError> {
        let light_code = make_color_code(color, d);

        match button {
            Button::GridButton { x, y } => {
                let button_code = y * 16 + x;
                self.send(&[0x90, button_code, light_code])?;
            }
            Button::ControlButton { index } => {
                let button_code = 104 + index;
                self.send(&[0xB0, button_code, light_code])?;
            }
        }

        Ok(())
    }

    /// In order to make maximum use of the original Launchpad's slow midi speeds, a rapid LED
    /// lighting mode was invented which allows the lighting of two leds in just a single message.
    /// To use this mode, simply start sending these message and the Launchpad will update the 8x8
    /// grid in left-to-right, top-to-bottom order, then the eight scene launch buttons in
    /// top-to-bottom order, and finally the eight Automap/Live buttons in left-to-right order
    /// (these are otherwise inaccessible using note-on messages). Overflowing data will be ignored.
    ///
    /// To leave the mode, simply send any other message. Sending another kind of message and then
    /// re-sending this message will reset the cursor to the top left of the grid.
    pub fn set_button_rapid(
        &mut self,
        color1: Color,
        dbb1: DoubleBufferingBehavior,
        color2: Color,
        dbb2: DoubleBufferingBehavior,
    ) -> Result<(), crate::MidiError> {
        self.send(&[
            0x92,
            make_color_code(color1, dbb1),
            make_color_code(color2, dbb2),
        ])
    }

    /// Turns on all LEDs to a certain brightness, dictated by the `brightness` parameter. According
    /// to the Launchpad documentation, sending this command resets various configuration settings -
    /// see `reset()` for more information. However, in my experience, that only sometimes happens.
    /// Weird.
    ///
    /// This function is primarily intended as a diagnostics tool to verify that the library and the
    /// device is working correctly.
    pub fn turn_on_all_leds(&mut self, brightness: Brightness) -> Result<(), crate::MidiError> {
        let brightness_code = match brightness {
            Brightness::Off => 0,
            Brightness::Low => 125,
            Brightness::Medium => 126,
            Brightness::Full => 127,
        };

        self.send(&[0xB0, 0, brightness_code])
    }

    /// Launchpad controls the brightness of its LEDs by continually switching them on and off
    /// faster than the eye can see: a technique known as multiplexing. This command provides a way
    /// of altering the proportion of time for which the LEDs are on while they are in low- and
    /// medium-brightness modes. This proportion is known as the duty cycle.
    ///
    /// Manipulating this is useful for fade effects, for adjusting contrast, and for creating
    /// custom palettes.
    ///
    /// The default duty cycle is 1/5 meaning that low-brightness LEDs are on for only every fifth
    /// multiplex pass, and medium-brightness LEDs are on for two passes in every five. Generally,
    /// lower duty cycles (numbers closer to zero) will increase contrast between different
    /// brightness settings but will also increase flicker; higher ones will eliminate flicker, but
    /// will also reduce contrast. Note that using less simple ratios (such as 3/17 or 2/11) can
    /// also increase perceived flicker.
    ///
    /// If you are particularly sensitive to strobing lights, please use this command with care when
    /// working with large areas of low-brightness LEDs: in particular, avoid duty cycles of 1/8 or
    /// less.
    pub fn set_duty_cycle(
        &mut self,
        numerator: u8,
        denominator: u8,
    ) -> Result<(), crate::MidiError> {
        assert!(numerator >= 1);
        assert!(numerator <= 16);
        assert!(denominator >= 3);
        assert!(denominator <= 18);

        if numerator < 9 {
            self.send(&[0xB0, 30, 16 * (numerator - 1) + (denominator - 3)])
        } else {
            self.send(&[0xB0, 31, 16 * (numerator - 9) + (denominator - 3)])
        }
    }

    /// This method controls the double buffering mode on the Launchpad. See the module
    /// documentation for an explanation on double buffering.
    ///
    /// The default state is no flashing; the first buffer is both the update and the displayed
    /// buffer: In this mode, any LED data written to Launchpad is displayed instantly. Sending this
    /// message also resets the flash timer, so it can be used to resynchronise the flash rates of
    /// all the Launchpads connected to a system.
    ///
    /// - If `copy` is set, copy the LED states from the new displayed buffer to the new updating
    ///   buffer.
    /// - If `flash` is set, continually flip displayed buffers to make selected LEDs flash.
    /// - `updated`: the new updated buffer
    /// - `displayed`: the new displayed buffer
    pub fn control_double_buffering(&mut self, d: DoubleBuffering) -> Result<(), crate::MidiError> {
        let last_byte = 0b00100000
            | ((d.copy as u8) << 4)
            | ((d.flash as u8) << 3)
            | ((d.edited_buffer as u8) << 2)
            | d.displayed_buffer as u8;

        self.send(&[0xB0, 0, last_byte])
    }

    pub fn request_device_inquiry(&mut self, query: DeviceIdQuery) -> Result<(), crate::MidiError> {
        request_device_inquiry(self, query)
    }

    pub fn request_version_inquiry(&mut self) -> Result<(), crate::MidiError> {
        request_version_inquiry(self)
    }

    pub fn scroll_text(
        &mut self,
        text: &[u8],
        color: Color,
        should_loop: bool,
    ) -> Result<(), crate::MidiError> {
        let color_code = make_color_code_loopable(color, should_loop);

        let bytes = &[&[240, 0, 32, 41, 9, color_code], text, &[247]].concat();

        self.send(bytes)
    }

    // -----------------------------
    // Shorthand functions:
    // -----------------------------

    /// All LEDs are turned off, and the mapping mode, buffer settings, and duty cycle are reset to
    /// their default values.
    pub fn reset(&mut self) -> Result<(), crate::MidiError> {
        self.turn_on_all_leds(Brightness::Off)
    }

    pub fn set_all_buttons(&mut self, color: Color, dbb: DoubleBufferingBehavior) -> Result<(), crate::MidiError> {
        for _ in 0..40 {
            self.set_button_rapid(color, dbb, color, dbb)?;
        }

        Ok(())
    }

    pub fn light(&mut self, button: Button, color: Color) -> Result<(), crate::MidiError> {
        self.set_button(button, color, DoubleBufferingBehavior::Copy)
    }

    pub fn light_all_rapid(&mut self, color: Color) -> Result<(), crate::MidiError> {
        self.set_all_buttons(color, DoubleBufferingBehavior::Copy)
    }
}
