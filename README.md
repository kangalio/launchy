# launchy
An exhaustive Rust API for the Novation Launchpad devices, optimized for maximum expressiveness and minimum boilerplate!

<a href="https://youtu.be/DHwv7yu5dJc"><img src="https://imgur.com/gBKAjgS.jpg" width="50%"/></a>

---

Launchy is a library for the Novation Launchpad MIDI devices, for the Rust programming language.

## Features

- the **Canvas API** provides a powerful and concise way to control your Launchpads, for games or small lightshows
- the **direct Input/Output API** provides absolute control over your device and fine-grained access to the entire MIDI API of your device
- it's possible to chain multiple Launchpads together and use them as if it was one single big device
- optional support for [`embedded-graphics`](https://github.com/jamwaffles/embedded-graphics)
- very modular design: it's very easy to add support for new devices, or to add new features to [`Canvas`]

## Supported devices
- [ ] Launchpad
- [x] Launchpad S
- [ ] Launchpad Mini
- [x] Launchpad Control
- [x] Launchpad Control XL
- [ ] Launchpad Pro
- [x] Launchpad MK2
- [ ] Launchpad X
- [ ] Launchpad Mini MK3
- [ ] Launchpad Pro MK2

## Canvas API
Launchy provides a `Canvas` trait that allows you to abstract over the hardware-specific details of your Launchpad and write concise, performant and 
Launchpad-agnostic code.

The `Canvas` API even allows you to chain multiple Launchpads together and use them as if they were a single device. See `CanvasLayout` for that.

## Direct Input/Output API
In cases where you need direct access to your device's API, the abstraction provided by the `Canvas` API gets in your way.

Say if you wanted to programmatically retrieve the firmware version of your Launchpad MK2:
```rust
let input = launchy::mk2::Input::guess_polling()?;
let mut output = launchy::mk2::Output::guess()?;

output.request_version_inquiry()?;
for msg in input.iter() {
	if let launchy::mk2::Message::VersionInquiry { firmware_version, .. } = msg {
		println!("The firmware version is {}", firmware_version);
	}
}
```

## Examples
### Satisfying pulse light effect
<a href="https://youtu.be/DHwv7yu5dJc"><img src="https://imgur.com/gBKAjgS.jpg" width="50%"/></a>

```rust
// Setup devices
let (mut canvas, poller) = launchy::CanvasLayout::new_polling();
canvas.add_by_guess_rotated::<launchy::control::Canvas>(0, 14, launchy::Rotation::Right)?;
canvas.add_by_guess_rotated::<launchy::mk2::Canvas>(10, 18, launchy::Rotation::UpsideDown)?;
canvas.add_by_guess_rotated::<launchy::s::Canvas>(2, 8, launchy::Rotation::Right)?;
let mut canvas = canvas.into_padded();

// Do the actual animation
for color in (0u64..).map(|f| Color::red_green_color(f as f32 / 60.0 / 2.5)) {
	for msg in poller.iter_for_millis(17).filter(|msg| msg.is_press()) {
		canvas[msg.pad()] = color * 60.0;
	}
	canvas.flush()?;

	for pad in canvas.iter() {
		let surrounding_color = pad.neighbors_5().iter()
				.map(|&p| canvas.get(p).unwrap_or(Color::BLACK))
				.sum::<Color>() / 5.0 / 1.05;
		
		canvas[pad] = canvas[pad].mix(surrounding_color, 0.4);
	}
}
```

<!--### Snake game
```rust
let (mut canvas, poller) = launchy::mk2::Canvas::guess_polling()?;

// Initialize snake
let mut snake = std::collections::VecDeque::new();
snake.push_front(Pad { x: 0, y: 1 });
snake.push_front(Pad { x: 1, y: 1 });
snake.push_front(Pad { x: 2, y: 1 });

// Set initial snake direction and pellet position
let mut direction = (1, 0);
let mut pellet = Pad { x: 5, y: 6 };

loop {
	for msg in poller.iter_for(Duration::from_millis(500)).filter(|msg| msg.is_press()) {
		match msg.pad() {
			Pad { x: 0, y: 0 } => direction = (0, -1),
			Pad { x: 1, y: 0 } => direction = (0, 1),
			Pad { x: 2, y: 0 } => direction = (-1, 0),
			Pad { x: 3, y: 0 } => direction = (1, 0),
			_ => {},
		}
	}

	if snake.contains(&(snake[0] + direction)) {
		break;
	} else {
		snake.push_front(snake[0] + direction);
	}

	if snake[0] == pellet {
		pellet = Pad { x: (rand::random() * 9) as i32, y: (rand::random() * 9) as i32 };
	} else {
		snake.pop_back();
	}

	canvas.clear();
	for &pad in &snake {
		canvas[pad] = Color::YELLOW;
	}
	canvas[pellet] = Color::GREEN;
	canvas.flush()?;
}
```-->

### Seamless text scrolling across multiple Launchpads (leveraging `embedded_graphics`)
<a href="https://youtu.be/BJqoH3p9mhE"><img src="https://imgur.com/Fxe9al9.jpg" width="50%"/></a>

(This image shows the first three letters of the word "Hello")

```rust
use embedded_graphics::{fonts::{Font6x8, Text}, prelude::{Drawable, Point}, style::TextStyle};

// Setup the Launchpad layout
let mut canvas = launchy::CanvasLayout::new(|_msg| {});
canvas.add_by_guess_rotated::<launchy::control::Canvas>(0, 14, launchy::Rotation::Right)?;
canvas.add_by_guess_rotated::<launchy::mk2::Canvas>(10, 18, launchy::Rotation::UpsideDown)?;
canvas.add_by_guess_rotated::<launchy::s::Canvas>(2, 8, launchy::Rotation::Right)?;

// Do the text scrolling
let mut x_offset = 19;
loop {
	canvas.clear();

	let t = Text::new("Hello world! :)", Point::new(x_offset, 3))
		.into_styled(TextStyle::new(Font6x8, Color::RED.into()))
		.draw(&mut canvas).unwrap();
	
	canvas.flush()?;

	sleep(100);
	x_offset -= 1;
}
```

## Why not just use [launch-rs](https://github.com/jamesmunns/launch-rs)?

- Last commit in 2017
- Only supports Launchpad MK2
- Only low-level access to the Launchpad is provided. There is no way to write high-level, concise interfacing code
- Uses the [PortMidi](https://github.com/musitdev/portmidi-rs) crate which is not as actively developed as [midir](https://github.com/Boddlnagg/midir), which Launchy uses
- Doesn't have any of the advanced features that Launchy provides
