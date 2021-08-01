# Low-level API

## InputDevice and OutputDevice traits

The `InputDevice` and `OutputDevice` traits provide common device-agnostic utilities.

Implementors of `InputDevice` just need to supply some device-specific strings and a function to
parse incoming MIDI messages into a device-specific Message type. In turn, `InputDevice` provides
constructor methods.

Note: `InputDevice`-provided constructor methods return a device-agnostic connection struct instead
of Self. This is done to abstract away as much common code from the individual backends as possible.

The `OutputDevice` trait doesn't have this peculiarity; its implementors store MIDI connections etc.
themselves. Therefore, implementors need to bring a constructor (`from_connection()`) and a MIDI
byte send function (`send()`) themselves. In turn, they get convenience constructors.

Each supported device has its own module and stores its `InputDevice` and `OutputDevice`
implementations in `input.rs` and `output.rs` respectively.

## Conventions

TODO
- shorthand vs "raw" functions
- convention of mapping functions 1:1 to MIDI messages
- chosen trade-off of supporting all layouts vs just X/Y layout
- Verbatim Message enum variant (which doesn't exist yet, currently just panic)
- shared Button80 struct and double buffering structs
- add an image of the Launchpad to the module root

# High-level Canvas API

## How it is implemented

The Canvas API is implemented by every struct that represents a grid of lightable buttons. This
includes every device, but also "virtual Launchpads" like `CanvasLayout`.

To reduce duplication, Canvas implementations for devices is handled by the device-agnostic
`DeviceCanvas<Spec: DeviceSpec>` struct. All device-specific properties are given via the
`DeviceTrait` in the generic parameter. For example the canvas struct for the Launchpad S
(`launchy::s::Canvas`) is actually just a type alias for `launchy::DeviceCanvas<launchy::s::Spec>`.

## What it provides

TODO
- Pad struct
  - Index<Pad> implementation
  - .iter()
- Color struct and how it loses precision for devices
- calibrating brightness across launchpads
