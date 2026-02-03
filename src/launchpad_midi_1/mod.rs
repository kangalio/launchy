/*!
# Launchpad MIDI 1 low-level API

![Picture](https://imgur.com/NzwlsRh.png)

## Launchpad MK1 Button Layout Notes

The original Launchpad (MIDI 1) features a physical layout that `launchy` maps as follows:
- **Main 8x8 Grid Pads:** Occupy absolute coordinates (x: 0-7, y: 1-8). These are typically used for primary interaction (e.g., playing notes, triggering clips).
- **Top Row Control Buttons (Automap/Live):** 8 circular buttons with absolute coordinates (x: 0-7, y: 0). These typically serve as menu or control functions.
- **Right Column Scene Launch Buttons:** 8 circular buttons with absolute coordinates (x: 8, y: 1-8). These also typically serve as menu or control functions (e.g., launching scenes).
- **Invalid Coordinate:** The absolute coordinate (x: 8, y: 0) does not correspond to a physical button on the Launchpad MK1.
*/

mod input;
pub use input::*;

mod output;
pub use output::*;

pub use crate::protocols::Button80 as Button;

#[doc(hidden)]
pub struct Spec;

impl crate::DeviceSpec for Spec {
    const BOUNDING_BOX_WIDTH: u32 = 9;
    const BOUNDING_BOX_HEIGHT: u32 = 9;
    const COLOR_PRECISION: u16 = 4;

    type Input = Input;
    type Output = Output;

    fn is_valid(x: u32, y: u32) -> bool {
        if x > 8 || y > 8 {
            return false;
        }
        if x == 8 && y == 0 {
            return false;
        }
        true
    }

    fn flush(
        canvas: &mut crate::DeviceCanvas<Self>,
        changes: &[(u32, u32, (u8, u8, u8))],
    ) -> Result<(), crate::MidiError> {
        use crate::Canvas;

        let convert_color = |color: crate::Color| {
            let (r, g, _b) = color.quantize(Self::COLOR_PRECISION as u8);
            Color::new(r, g)
        };

        // Because rapid-update mode lets us set 2 LEDs per instruction, if we
        // have more than 40 updates, it's faster to use rapid-update mode to
        // re-write the whole canvas
        if changes.len() > 40 {
            // Set the main body
            for y in 1..=8 {
                for x in (0..=7).step_by(2) {
                    canvas.output.set_button_rapid(
                        convert_color(*canvas.low_level_get_pending(x, y).unwrap()),
                        DoubleBufferingBehavior::Copy,
                        convert_color(*canvas.low_level_get_pending(x + 1, y).unwrap()),
                        DoubleBufferingBehavior::Copy,
                    )?;
                }
            }

            // Set the scene launch buttons (x = 8)
            for y in (1..=8).step_by(2) {
                canvas.output.set_button_rapid(
                    convert_color(*canvas.low_level_get_pending(8, y).unwrap()),
                    DoubleBufferingBehavior::Copy,
                    convert_color(*canvas.low_level_get_pending(8, y + 1).unwrap()),
                    DoubleBufferingBehavior::Copy,
                )?;
            }

            // Set the Automap/live buttons (y = 0)
            for x in (0..=7).step_by(2) {
                canvas.output.set_button_rapid(
                    convert_color(*canvas.low_level_get_pending(x, 0).unwrap()),
                    DoubleBufferingBehavior::Copy,
                    convert_color(*canvas.low_level_get_pending(x + 1, 0).unwrap()),
                    DoubleBufferingBehavior::Copy,
                )?;
            }

            // dummy-light some button just to get out of the rapid update mode
            canvas.output.light(
                Button::ControlButton { index: 0 },
                convert_color(*canvas.low_level_get_pending(0, 0).unwrap()),
            )?;
        } else {
            for &(x, y, (r, g, _b)) in changes {
                canvas
                    .output
                    .light(Button::from_abs(x as u8, y as u8), Color::new(r, g))?;
            }
        }

        Ok(())
    }

    fn convert_message(msg: Message) -> Option<crate::CanvasMessage> {
        match msg {
            Message::Press { button } => Some(crate::CanvasMessage::Press {
                x: button.abs_x() as u32,
                y: button.abs_y() as u32,
            }),
            Message::Release { button } => Some(crate::CanvasMessage::Release {
                x: button.abs_x() as u32,
                y: button.abs_y() as u32,
            }),
            Message::TextEndedOrLooped
            | Message::UnknownShortMessage { .. }
            | Message::DeviceInquiry(_)
            | Message::VersionInquiry(_) => None,
        }
    }

    fn setup(output: &mut Self::Output) -> Result<(), crate::MidiError> {
        output.reset()?;
        Ok(())
    }
}

pub type Canvas<'a> = crate::DeviceCanvas<Spec>;
