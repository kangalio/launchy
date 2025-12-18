# Launchpad MIDI 1 (MK1) Support

This module provides support for the original Novation Launchpad, also known as the Launchpad MK1. This device has a similar protocol to the Launchpad S, but with some key differences, most notably a slower processing speed.

## Overview

The Launchpad MK1 uses a MIDI-based protocol for communication. This module implements the necessary logic to interface with the device, allowing you to control the LEDs and handle button presses.

Due to the hardware limitations of the MK1, there are some performance considerations to keep in mind. Rapidly sending MIDI messages to the device can cause it to become unresponsive. This module attempts to mitigate this by introducing a small delay between messages, but it is still possible to overwhelm the device if you are not careful.

## Usage

To use this module, you can create a `Device` instance and use the `Canvas` trait to control the LEDs. Here is a basic example:

```rust
use launchy::midi1::{Device, Canvas, Button, Color};

fn main() -> Result<(), launchy::errors::Error> {
    let mut device = Device::from_keyword("Launchpad MIDI")?;
    
    // Turn all LEDs on
    device.fill(Color::RED);
    
    // Wait for a button press
    for message in device.iter() {
        if let Message::Press { button } = message {
            println!("Pressed button: {:?}", button);
            break;
        }
    }
    
    // Clear the LEDs
    device.clear();
    
    Ok(())
}
```
