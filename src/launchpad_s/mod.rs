/*!
# Launchpad S low-level API

![Picture](https://imgur.com/NzwlsRh.png)
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
            let (r, g, _b) = color.quantize(Self::COLOR_PRECISION);
            Color::new(r, g)
        };

        if changes.len() > 41 {
            for y in 1..=8 {
                for x in (0..=7).step_by(2) {
                    canvas.output.set_button_rapid(
                        convert_color(canvas.get_new_unchecked(x, y)),
                        DoubleBufferingBehavior::Copy,
                        convert_color(canvas.get_new_unchecked(x + 1, y)),
                        DoubleBufferingBehavior::Copy,
                    )?;
                }
            }

            // dummy-light some button just to get out of the rapid update mode
            canvas.output.light(
                Button::ControlButton { index: 0 },
                convert_color(canvas.get_new_unchecked(0, 0)),
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
            _ => None,
        }
    }

    fn setup(output: &mut Self::Output) -> Result<(), crate::MidiError> {
        output.reset()?;
        Ok(())
    }
}

pub type Canvas<'a> = crate::DeviceCanvas<Spec>;
