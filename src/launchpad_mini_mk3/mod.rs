/*!
# Launchpad Mini MK3 low-level API

![Picture](https://imgur.com/ra7nOzc.png)

The Launchpad Mini MK3 has a 9x9 grid with 80 buttons. The top
row are control buttons, indexed `0..7`. The `launchy` library
considers the right-hand side buttons to be part of the grid,
so the grid has a size of 9x8.
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
    const COLOR_PRECISION: u16 = 128;

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
        let changes = changes.iter().map(|&(x, y, (r, g, b))| {
            let color = RgbColor::new(r, g, b);

            let button = Button::from_abs(x as u8, y as u8);

            (button, color)
        });
        canvas.output.light_multiple_rgb(changes)
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
            Message::ApplicationVersion(_)
            | Message::BootloaderVersion(_)
            | Message::SleepMode(_)
            | Message::Brightness(_)
            | Message::ChangeLayout(_) => None,
        }
    }
}

pub type Canvas<'a> = crate::DeviceCanvas<Spec>;
