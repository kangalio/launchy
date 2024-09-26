/*!
# Launchpad Mini low-level API

![Picture](https://imgur.com/sdLy3XK.png)
*/

mod input;
pub use input::*;

mod output;
pub use output::*;

use crate::prelude::{LogicalButton, PhysicalButton};
pub use crate::protocols::LogicalButton as Button;
use crate::shared::{default_logical_to_physical, default_physical_to_logical};

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

    fn to_physical(button: LogicalButton) -> PhysicalButton {
        default_logical_to_physical(button)
    }

    fn to_logical(button: PhysicalButton) -> Option<LogicalButton> {
        default_physical_to_logical::<Self>(button)
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
                if let Some(b) = Self::to_logical(PhysicalButton::new(x, y)) {
                    canvas.output.light(b, Color::new(r, g))?;
                }
            }
        }

        Ok(())
    }

    fn convert_message(msg: Message) -> Option<crate::CanvasMessage> {
        match msg {
            Message::Press { button } => {
                let b = Self::to_physical(button);
                Some(crate::CanvasMessage::Press { x: b.x, y: b.y })
            }
            Message::Release { button } => {
                let b = Self::to_physical(button);
                Some(crate::CanvasMessage::Release { x: b.x, y: b.y })
            }
            Message::TextEndedOrLooped => None,
            Message::DeviceInquiry(_) => None,
            Message::VersionInquiry(_) => None,
        }
    }
}

pub type Canvas<'a> = crate::DeviceCanvas<Spec>;
