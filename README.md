# launchy
An exhaustive Rust API for the Novation Launchpad devices, optimized for maximum expressiveness and minimum boilerplate!

<!--DEMO VIDEO HERE-->

---

Launchy is a library for the Novation Launchpad MIDI devices, written in Rust.

- the **Canvas API** provides a powerful and concise way to control your Launchpads, for games or small lightshows
- the **direct Input/Output API** provides absolute control over your device and fine-grained access to the entire MIDI API of your device

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

## Why not just use [launch-rs](https://github.com/jamesmunns/launch-rs)?

- Last commit in 2017
- Only supports Launchpad MK2
- Uses the [PortMidi](https://github.com/musitdev/portmidi-rs) crate which is not as actively developed as [midir](https://github.com/Boddlnagg/midir), which Launchy uses
- Only low-level access to the Launchpad is provided. There is no way to write high-level, concise interfacing code
