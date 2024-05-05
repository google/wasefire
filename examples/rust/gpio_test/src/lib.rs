// Copyright 2023 Google LLC
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

//! Tests that the GPIOs are working properly.
//!
//! This test assumes that GPIO 0, 1, and 2 are connected together.

#![no_std]
wasefire::applet!();

use alloc::vec::Vec;

use wasefire::gpio::InputConfig::*;
use wasefire::gpio::OutputConfig::*;
use wasefire::gpio::{Config, Gpio, InputConfig, OutputConfig};

fn main() -> Result<(), Error> {
    if gpio::count() < 3 {
        debug!("This test needs at least 3 GPIOs (connected together).");
        debug::exit(true);
    }
    let mut configs = Vec::new();
    for input in [InputConfig::Disabled, Floating, PullUp, PullDown] {
        for output in [OutputConfig::Disabled, PushPull, OpenDrain, OpenSource] {
            match (input, output) {
                (InputConfig::Disabled, PushPull) => (),
                (_, PushPull) => continue,
                (PullUp, OpenSource) | (PullDown, OpenDrain) => continue,
                _ => (),
            }
            for initial in [false, true] {
                configs.push(Config::new(input, output, initial));
                if output == OutputConfig::Disabled {
                    break;
                }
            }
        }
    }
    for src in &configs {
        for dst in &configs {
            let expected = match simulate(src, dst) {
                Some(x) => x,
                None => continue,
            };
            debug!("{} + {} = {expected:?}", Pretty(src), Pretty(dst));
            let _src = Gpio::new(0, src)?;
            let dst = Gpio::new(1, dst)?;
            match expected {
                Some(expected) => debug::assert_eq(&dst.read()?, &expected),
                None => {
                    let bus = Gpio::new(2, &Config::output(PushPull, false))?;
                    debug::assert_eq(&dst.read()?, &false);
                    bus.write(true)?;
                    debug::assert_eq(&dst.read()?, &true);
                }
            }
        }
    }
    debug::exit(true);
}

fn simulate(src: &Config, dst: &Config) -> Option<Option<bool>> {
    if src.output == OutputConfig::Disabled || dst.input == InputConfig::Disabled {
        return None;
    }
    let mut state = State::default();
    state.update(src);
    state.update(dst);
    state.eval()
}

#[derive(Default)]
struct State {
    pull_low: bool,
    pull_high: bool,
    drive_low: bool,
    drive_high: bool,
}

impl State {
    fn update(&mut self, config: &Config) {
        match config.input {
            PullDown => self.pull_low = true,
            PullUp => self.pull_high = true,
            _ => (),
        }
        match (config.output, config.initial) {
            (OpenDrain | PushPull, false) => self.drive_low = true,
            (OpenSource | PushPull, true) => self.drive_high = true,
            _ => (),
        }
    }

    fn eval(&self) -> Option<Option<bool>> {
        let drive = combine(self.drive_low, self.drive_high)?;
        let pull = combine(self.pull_low, self.pull_high)?;
        Some(drive.or(pull))
    }
}

fn combine(low: bool, high: bool) -> Option<Option<bool>> {
    match (low, high) {
        (true, true) => None,
        (true, false) => Some(Some(false)),
        (false, true) => Some(Some(true)),
        (false, false) => Some(None),
    }
}

struct Pretty<'a>(&'a Config);

impl<'a> core::fmt::Display for Pretty<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Config { input, output, initial } = &self.0;
        match (input, output) {
            (InputConfig::Disabled, OutputConfig::Disabled) => write!(f, "Disconnected"),
            (_, OutputConfig::Disabled) => write!(f, "{input:?}"),
            (InputConfig::Disabled, _) => write!(f, "{output:?}({initial})"),
            (_, _) => write!(f, "{input:?}-{output:?}({initial})"),
        }
    }
}
