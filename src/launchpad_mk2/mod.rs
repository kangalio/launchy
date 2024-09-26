/*!
# Launchpad MK2 low-level API

![Picture](https://imgur.com/PXeHwre.png)
*/

mod input;
pub use input::*;

mod output;
pub use output::*;

pub use crate::protocols::LogicalButton as Button;
use crate::{
    prelude::PhysicalButton,
    shared::{default_logical_to_physical, default_physical_to_logical},
};

#[doc(hidden)]
pub struct Spec;

impl crate::DeviceSpec for Spec {
    const BOUNDING_BOX_WIDTH: u32 = 9;
    const BOUNDING_BOX_HEIGHT: u32 = 9;
    const COLOR_PRECISION: u16 = 64;

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

    fn to_physical(button: Button) -> PhysicalButton {
        default_logical_to_physical(button)
    }

    fn to_logical(button: PhysicalButton) -> Option<Button> {
        default_physical_to_logical::<Self>(button)
    }

    fn flush(
        canvas: &mut crate::DeviceCanvas<Self>,
        changes: &[(u32, u32, (u8, u8, u8))],
    ) -> Result<(), crate::MidiError> {
        let changes = changes.iter().filter_map(|&(x, y, (r, g, b))| {
            let color = RgbColor::new(r, g, b);

            Self::to_logical(PhysicalButton::new(x, y)).map(|b| (b, color))
        });
        canvas.output.light_multiple_rgb(changes)
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
            Message::TextEndedOrLooped
            | Message::DeviceInquiry(_)
            | Message::VersionInquiry(_)
            | Message::FaderChange { .. } => None,
        }
    }
}

pub type Canvas<'a> = crate::DeviceCanvas<Spec>;
