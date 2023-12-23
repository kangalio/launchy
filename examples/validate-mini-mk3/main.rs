//! Validation suite for the Mini MK3 direct API
//!
//! This exercises all supported features.
use std::io::{stdin, stdout, Write};

use launchy::{
    mini_mk3::{Button, Message, PaletteColor, RgbColor, SleepMode},
    s::DeviceIdQuery,
    Canvas, Color, InputDevice, MidiError, MsgPollingWrapper, OutputDevice, Pad,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Show all interface names.
    let midi = midir::MidiOutput::new("launchy")?;
    for port in midi.ports() {
        println!("{}", midi.port_name(&port)?);
    }

    // Load MK3
    let mut mk3 = launchy::mini_mk3::Output::guess()?;
    mk3.stop_scroll()?;

    let mut failed_tests: Vec<String> = vec![];

    macro_rules! test {
        ($description:literal, $test:block) => {
            print!("{} [Y/n] ", $description);
            stdout().flush()?;
            $test;
            if !yes_no()? {
                failed_tests.push($description.into());
            }
        };
    }

    println!("/////////////////////////////////////////////");
    println!("//  OUTPUT TESTS");
    println!("/////////////////////////////////////////////");

    test!("All lights turn dark grey", {
        mk3.light_all(PaletteColor::DARK_GRAY)?;
    });

    mk3.light_all(PaletteColor::BLACK)?;

    test!("Control button 0 turns palette red", {
        mk3.light(Button::ControlButton { index: 0 }, PaletteColor::RED)?;
    });
    test!("Control button 1 turns an RGB color purple", {
        mk3.light_rgb(
            Button::ControlButton { index: 1 },
            RgbColor::new(127, 0, 127),
        )?;
    });
    test!("Grid button (0, 0) turns palette red", {
        mk3.light(Button::GridButton { x: 0, y: 0 }, PaletteColor::new(5))?;
    });
    test!("Grid button (1, 0) turns RGB purple", {
        mk3.light_rgb(
            Button::GridButton { x: 1, y: 0 },
            RgbColor::new(127, 0, 127),
        )?;
    });
    test!("Grid buttons (0, 1) and (1, 1) turn palette yellow", {
        mk3.light_multiple([
            (Button::GridButton { x: 0, y: 1 }, PaletteColor::YELLOW),
            (Button::GridButton { x: 1, y: 1 }, PaletteColor::YELLOW),
        ])?;
    });
    test!(
        "Grid buttons (0, 2) and (1, 2) turn palette white and light purple",
        {
            mk3.light_multiple_rgb([
                (
                    Button::GridButton { x: 0, y: 2 },
                    RgbColor::new(127, 127, 127),
                ),
                (
                    Button::GridButton { x: 1, y: 2 },
                    RgbColor::new(127, 80, 127),
                ),
            ])?;
        }
    );
    test!("Control button 2 flashes red", {
        mk3.flash(Button::ControlButton { index: 2 }, PaletteColor::RED)?;
    });
    test!("Control button 3 pulses red", {
        mk3.pulse(Button::ControlButton { index: 3 }, PaletteColor::RED)?;
    });
    test!("Control buttons 4 and 5 flash red", {
        mk3.flash_multiple([
            (Button::ControlButton { index: 4 }, PaletteColor::RED),
            (Button::ControlButton { index: 5 }, PaletteColor::RED),
        ])?;
    });
    test!("Control buttons 6 and 7 pulse red", {
        mk3.pulse_multiple([
            (Button::ControlButton { index: 6 }, PaletteColor::RED),
            (Button::ControlButton { index: 7 }, PaletteColor::RED),
        ])?;
    });
    test!("Grid button (2, 0) flashes red", {
        mk3.flash(Button::GridButton { x: 2, y: 0 }, PaletteColor::RED)?;
    });
    test!("Grid button (3, 0) pulses red", {
        mk3.pulse(Button::GridButton { x: 3, y: 0 }, PaletteColor::RED)?;
    });
    test!("Grid buttons (2, 1) and (2, 2) flash red", {
        mk3.flash_multiple([
            (Button::GridButton { x: 2, y: 1 }, PaletteColor::RED),
            (Button::GridButton { x: 2, y: 2 }, PaletteColor::RED),
        ])?;
    });
    test!("Grid buttons (3, 1) and (3, 2) pulse red", {
        mk3.pulse_multiple([
            (Button::GridButton { x: 3, y: 1 }, PaletteColor::RED),
            (Button::GridButton { x: 3, y: 2 }, PaletteColor::RED),
        ])?;
    });
    test!("Blue text scrolls across the pad", {
        mk3.scroll_text(b"Hello, world!", PaletteColor::BLUE, 10, true)?;
    });
    test!("The scroll has stopped", {
        mk3.stop_scroll()?;
    });
    test!("Grid rows 6 and 7 turn yellow", {
        mk3.light_rows([(6, PaletteColor::YELLOW), (7, PaletteColor::YELLOW)])?;
    });
    test!("Grid columns 6 and 7 turn green", {
        mk3.light_columns([(6, PaletteColor::GREEN), (7, PaletteColor::GREEN)])?;
    });
    test!("Brightness is dimmed", { mk3.set_brightness(20)? });
    test!("Brightness is restored", { mk3.set_brightness(127)? });
    test!("LEDs are off", { mk3.send_sleep(SleepMode::Sleep)? });
    test!("LEDs are restored", { mk3.send_sleep(SleepMode::Wake)? });

    println!("/////////////////////////////////////////////");
    println!("//  INPUT TESTS");
    println!("/////////////////////////////////////////////");

    let input = launchy::mini_mk3::Input::guess_polling()?;

    mk3.request_sleep_mode()?;
    for msg in input.iter() {
        if let Message::SleepMode(mode) = msg {
            println!("SleepMode {mode:?}");
            break;
        }
    }

    mk3.request_brightness()?;
    for msg in input.iter() {
        if let Message::Brightness(b) = msg {
            println!("Brightness {b:?}");
            break;
        }
    }

    mk3.request_device_inquiry(DeviceIdQuery::Any)?;
    for msg in input.iter() {
        match msg {
            Message::BootloaderVersion(version) => {
                println!("The boot loader version is {:?}", version.bytes);
                break;
            }
            Message::ApplicationVersion(version) => {
                println!("The application version is {:?}", version.bytes);
                break;
            }
            _ => println!("{msg:?}"),
        }
    }

    println!("Press and release the PURPLE button at (4, 3)");
    mk3.light(Button::grid(4, 3), PaletteColor::PURPLE)?;

    let mut did_see_press = false;
    for msg in input.iter() {
        match msg {
            Message::Press(button) if button == Button::grid(4, 3) => {
                println!("Press");
                did_see_press = true;
            }
            Message::Release(button) if button == Button::grid(4, 3) => {
                println!("Release");
                if did_see_press {
                    break;
                }
            }
            _ => (),
        }
    }

    println!("Press buttons on the screen to observe responses; top-left button exits.");
    println!("(Be sure to try all buttons on the edges)");
    do_canvas_demo()?;

    println!("All tests done!");

    if !failed_tests.is_empty() {
        println!("/////////////////////////////////////////////");
        println!("//  FAILED TESTS");
        println!("/////////////////////////////////////////////");
        for failed_test in failed_tests {
            println!("- {}", failed_test);
        }
    }

    Ok(())
}

fn yes_no() -> Result<bool, std::io::Error> {
    let mut answer = String::new();
    loop {
        answer.clear();
        stdin().read_line(&mut answer)?;
        let buf = answer.trim_end().to_lowercase();
        if buf.is_empty() || buf == "y" {
            return Ok(true);
        }
        if buf == "n" {
            return Ok(false);
        }
        println!("[Y/n] ");
    }
}

fn do_canvas_demo() -> Result<(), MidiError> {
    // Setup devices
    let (mut canvas, poller) = launchy::CanvasLayout::new_polling();
    canvas.add_by_guess::<launchy::mini_mk3::Canvas>(0, 0)?;
    let mut canvas = canvas.into_padded();

    // Do the actual animation. Top-left button stops.
    for color in (0u64..).map(|f| Color::red_green_color(f as f32 / 60.0 / 2.5)) {
        for msg in poller.iter_for_millis(17).filter(|msg| msg.is_press()) {
            println!("Press {:?}", msg.pad());
            canvas[msg.pad()] = color * 60.0;

            if msg.x() == 0 && msg.y() == 0 {
                return Ok(());
            }
        }
        canvas.flush()?;

        for pad in canvas.iter() {
            let surrounding_color = pad
                .neighbors_5()
                .iter()
                .map(|&p| canvas.get(p).unwrap_or(Color::BLACK))
                .sum::<Color>()
                / 5.0
                / 1.05;

            canvas[pad] = canvas[pad].mix(surrounding_color, 0.4);
        }
        canvas[Pad { x: 0, y: 0 }] = Color::RED;
    }

    Ok(())
}
