// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Provides control over GPIOs using buttons and LEDs.
//!
//! We don't support listeners on GPIOs yet so we wake-up every 100ms to check them and turn the LED
//! on or off accordingly.

#![no_std]
wasefire::applet!();

use wasefire::gpio::InputConfig::Floating;
use wasefire::gpio::OutputConfig::PushPull;
use wasefire::gpio::{Config, Gpio};
use wasefire::led::Status::{Off, On};

fn main() -> Result<(), Error> {
    let gpio_count = gpio::count();
    let led_count = led::count();
    let button_count = button::count();
    debug!("We have {gpio_count} GPIOs.");
    debug!("We have {led_count} LEDs.");
    debug!("We have {button_count} buttons.");

    let input_config = Config::input(Floating);
    let output_config = Config::output(PushPull, false);

    let mut led_index = 0;
    let mut button_index = 0;
    for gpio_index in 0 .. gpio_count {
        match (gpio_index.is_multiple_of(2), led_index < led_count, button_index < button_count) {
            (true, true, _) | (false, true, false) => {
                debug!("Mapping gpio {gpio_index} to LED {led_index}.");
                gpio::Listener::new(gpio_index, gpio::Event::AnyChange, {
                    let gpio = Gpio::new(gpio_index, &input_config)?;
                    move || led::set(led_index, if gpio.read().unwrap() { On } else { Off })
                })?
                .leak();
                led_index += 1;
            }
            (true, false, true) | (false, _, true) => {
                debug!("Mapping gpio {gpio_index} to button {button_index}.");
                button::Listener::new(button_index, {
                    let gpio = Gpio::new(gpio_index, &output_config)?;
                    move |state| gpio.write(matches!(state, button::State::Pressed)).unwrap()
                })?
                .leak();
                button_index += 1;
            }
            (_, false, false) => break,
        }
    }
    assert!(led_index != 0 || button_index != 0, "Nothing to do.");
    Ok(())
}
