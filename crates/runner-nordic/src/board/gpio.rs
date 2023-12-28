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

use embedded_hal::digital::v2::{InputPin, OutputPin, PinState};
use nrf52840_hal::gpio::{
    Disconnected, Floating, Input, Level, OpenDrain, OpenDrainConfig, OpenDrainIO, Output, Pin,
    PullDown, PullUp, PushPull,
};
use nrf52840_hal::pac;
use nrf52840_hal::pac::generic::Reg;
use nrf52840_hal::pac::p0::pin_cnf::PIN_CNF_SPEC;
use wasefire_board_api::gpio::{Api, Config, InputConfig, OutputConfig};
use wasefire_board_api::{Error, Id, Support};

use crate::with_state;

pub enum Impl {}

impl Support<usize> for Impl {
    const SUPPORT: usize = 4;
}

impl Api for Impl {
    fn configure(gpio: Id<Self>, config: Config) -> Result<(), Error> {
        with_state(|state| {
            let gpio = &mut state.gpios[*gpio];
            *gpio = match core::mem::replace(gpio, Gpio::Invalid) {
                Gpio::Invalid => unreachable!(),
                Gpio::Disconnected(x) => configure(x, config)?,
                Gpio::InputFloating(x) => configure(x, config)?,
                Gpio::InputPullDown(x) => configure(x, config)?,
                Gpio::InputPullUp(x) => configure(x, config)?,
                Gpio::OutputPushPull(x) => configure(x, config)?,
                Gpio::OutputOpenDrain(x) => configure(x, config)?,
                Gpio::OutputOpenDrainIO(x) => configure(x, config)?,
            };
            Ok(())
        })
    }

    fn read(gpio: Id<Self>) -> Result<bool, Error> {
        with_state(|state| {
            let gpio = &state.gpios[*gpio];
            match gpio {
                Gpio::Invalid => unreachable!(),
                Gpio::Disconnected(_) | Gpio::OutputPushPull(_) | Gpio::OutputOpenDrain(_) => {
                    return Err(Error::User);
                }
                Gpio::InputFloating(x) => x.is_high(),
                Gpio::InputPullDown(x) => x.is_high(),
                Gpio::InputPullUp(x) => x.is_high(),
                Gpio::OutputOpenDrainIO(x) => x.is_high(),
            }
            .map_err(|_| Error::World)
        })
    }

    fn write(gpio: Id<Self>, value: bool) -> Result<(), Error> {
        let value = if value { PinState::High } else { PinState::Low };
        with_state(|state| {
            let gpio = &mut state.gpios[*gpio];
            match gpio {
                Gpio::Invalid => unreachable!(),
                Gpio::Disconnected(_)
                | Gpio::InputFloating(_)
                | Gpio::InputPullDown(_)
                | Gpio::InputPullUp(_) => return Err(Error::User),
                Gpio::OutputPushPull(x) => x.set_state(value),
                Gpio::OutputOpenDrain(x) => x.set_state(value),
                Gpio::OutputOpenDrainIO(x) => x.set_state(value),
            }
            .map_err(|_| Error::World)
        })
    }
}

pub enum Gpio {
    Invalid,
    Disconnected(Pin<Disconnected>),
    InputFloating(Pin<Input<Floating>>),
    InputPullDown(Pin<Input<PullDown>>),
    InputPullUp(Pin<Input<PullUp>>),
    OutputPushPull(Pin<Output<PushPull>>),
    OutputOpenDrain(Pin<Output<OpenDrain>>),
    OutputOpenDrainIO(Pin<Output<OpenDrainIO>>),
}

impl Gpio {
    pub fn new(pin: Pin<Disconnected>) -> Self {
        Gpio::Disconnected(pin)
    }
}

fn configure<MODE>(pin: Pin<MODE>, config: Config) -> Result<Gpio, Error> {
    let initial = if config.initial { Level::High } else { Level::Low };
    Ok(match (config.input, config.output) {
        (InputConfig::Disabled, OutputConfig::Disabled) => {
            Gpio::Disconnected(pin.into_disconnected())
        }
        (InputConfig::Floating, OutputConfig::Disabled) => {
            Gpio::InputFloating(pin.into_floating_input())
        }
        (InputConfig::PullDown, OutputConfig::Disabled) => {
            Gpio::InputPullDown(pin.into_pulldown_input())
        }
        (InputConfig::PullUp, OutputConfig::Disabled) => Gpio::InputPullUp(pin.into_pullup_input()),
        (InputConfig::Disabled, OutputConfig::PushPull) => {
            Gpio::OutputPushPull(pin.into_push_pull_output(initial))
        }
        (_, OutputConfig::PushPull) => Err(Error::User)?,
        (InputConfig::Disabled, OutputConfig::OpenDrain) => Gpio::OutputOpenDrain(
            pin.into_open_drain_output(OpenDrainConfig::Standard0Disconnect1, initial),
        ),
        (InputConfig::Floating, OutputConfig::OpenDrain) => Gpio::OutputOpenDrainIO(
            pin.into_open_drain_input_output(OpenDrainConfig::Standard0Disconnect1, initial),
        ),
        (InputConfig::PullDown, OutputConfig::OpenDrain) => Err(Error::User)?,
        (InputConfig::PullUp, OutputConfig::OpenDrain) => {
            let pin =
                pin.into_open_drain_input_output(OpenDrainConfig::Standard0Disconnect1, initial);
            pin_cnf(&pin).modify(|_, w| w.pull().pullup());
            Gpio::OutputOpenDrainIO(pin)
        }
        (InputConfig::Disabled, OutputConfig::OpenSource) => Gpio::OutputOpenDrain(
            pin.into_open_drain_output(OpenDrainConfig::Disconnect0Standard1, initial),
        ),
        (InputConfig::Floating, OutputConfig::OpenSource) => Gpio::OutputOpenDrainIO(
            pin.into_open_drain_input_output(OpenDrainConfig::Disconnect0Standard1, initial),
        ),
        (InputConfig::PullDown, OutputConfig::OpenSource) => {
            let pin =
                pin.into_open_drain_input_output(OpenDrainConfig::Disconnect0Standard1, initial);
            pin_cnf(&pin).modify(|_, w| w.pull().pulldown());
            Gpio::OutputOpenDrainIO(pin)
        }
        (InputConfig::PullUp, OutputConfig::OpenSource) => Err(Error::User)?,
    })
}

fn pin_cnf(pin: &Pin<Output<OpenDrainIO>>) -> &Reg<PIN_CNF_SPEC> {
    let port = match pin.port() {
        nrf52840_hal::gpio::Port::Port0 => pac::P0::ptr(),
        nrf52840_hal::gpio::Port::Port1 => pac::P1::ptr(),
    };
    unsafe { &(*port).pin_cnf[pin.pin() as usize] }
}
