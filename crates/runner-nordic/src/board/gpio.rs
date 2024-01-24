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
use wasefire_error::Code;

use crate::with_state;

pub enum Impl {}

impl Support<usize> for Impl {
    const SUPPORT: usize = 4;
}

impl Api for Impl {
    fn configure(gpio: Id<Self>, config: Config) -> Result<(), Error> {
        with_state(|state| {
            let gpio = &mut state.gpios[*gpio];
            let (pin, res) = match core::mem::replace(gpio, Gpio::Invalid) {
                Gpio::Invalid => unreachable!(),
                Gpio::Disconnected(x) => configure(Gpio::Disconnected, x, config),
                Gpio::InputFloating(x) => configure(Gpio::InputFloating, x, config),
                Gpio::InputPullDown(x) => configure(Gpio::InputPullDown, x, config),
                Gpio::InputPullUp(x) => configure(Gpio::InputPullUp, x, config),
                Gpio::OutputPushPull(x) => configure(Gpio::OutputPushPull, x, config),
                Gpio::OutputOpenDrain(x) => configure(Gpio::OutputOpenDrain, x, config),
                Gpio::OutputOpenDrainIO(x) => configure(Gpio::OutputOpenDrainIO, x, config),
            };
            *gpio = pin;
            res
        })
    }

    fn read(gpio: Id<Self>) -> Result<bool, Error> {
        with_state(|state| {
            match &state.gpios[*gpio] {
                Gpio::Invalid => unreachable!(),
                Gpio::Disconnected(_) | Gpio::OutputPushPull(_) | Gpio::OutputOpenDrain(_) => {
                    return Err(Error::user(Code::InvalidState));
                }
                Gpio::InputFloating(x) => x.is_high(),
                Gpio::InputPullDown(x) => x.is_high(),
                Gpio::InputPullUp(x) => x.is_high(),
                Gpio::OutputOpenDrainIO(x) => x.is_high(),
            }
            .map_err(|_| Error::world(0))
        })
    }

    fn write(gpio: Id<Self>, value: bool) -> Result<(), Error> {
        let value = if value { PinState::High } else { PinState::Low };
        with_state(|state| {
            match &mut state.gpios[*gpio] {
                Gpio::Invalid => unreachable!(),
                Gpio::Disconnected(_)
                | Gpio::InputFloating(_)
                | Gpio::InputPullDown(_)
                | Gpio::InputPullUp(_) => return Err(Error::user(Code::InvalidState)),
                Gpio::OutputPushPull(x) => x.set_state(value),
                Gpio::OutputOpenDrain(x) => x.set_state(value),
                Gpio::OutputOpenDrainIO(x) => x.set_state(value),
            }
            .map_err(|_| Error::world(0))
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

fn configure<MODE>(
    wrap: impl FnOnce(Pin<MODE>) -> Gpio, pin: Pin<MODE>, config: Config,
) -> (Gpio, Result<(), Error>) {
    let initial = if config.initial { Level::High } else { Level::Low };
    match (config.input, config.output) {
        (InputConfig::Disabled, OutputConfig::Disabled) => {
            (Gpio::Disconnected(pin.into_disconnected()), Ok(()))
        }
        (InputConfig::Floating, OutputConfig::Disabled) => {
            (Gpio::InputFloating(pin.into_floating_input()), Ok(()))
        }
        (InputConfig::PullDown, OutputConfig::Disabled) => {
            (Gpio::InputPullDown(pin.into_pulldown_input()), Ok(()))
        }
        (InputConfig::PullUp, OutputConfig::Disabled) => {
            (Gpio::InputPullUp(pin.into_pullup_input()), Ok(()))
        }
        (InputConfig::Disabled, OutputConfig::PushPull) => {
            (Gpio::OutputPushPull(pin.into_push_pull_output(initial)), Ok(()))
        }
        (_, OutputConfig::PushPull) => (wrap(pin), Err(Error::user(0))),
        (InputConfig::Disabled, OutputConfig::OpenDrain) => (
            Gpio::OutputOpenDrain(
                pin.into_open_drain_output(OpenDrainConfig::Standard0Disconnect1, initial),
            ),
            Ok(()),
        ),
        (InputConfig::Floating, OutputConfig::OpenDrain) => (
            Gpio::OutputOpenDrainIO(
                pin.into_open_drain_input_output(OpenDrainConfig::Standard0Disconnect1, initial),
            ),
            Ok(()),
        ),
        (InputConfig::PullDown, OutputConfig::OpenDrain) => (wrap(pin), Err(Error::user(0))),
        (InputConfig::PullUp, OutputConfig::OpenDrain) => {
            let pin =
                pin.into_open_drain_input_output(OpenDrainConfig::Standard0Disconnect1, initial);
            pin_cnf(&pin).modify(|_, w| w.pull().pullup());
            (Gpio::OutputOpenDrainIO(pin), Ok(()))
        }
        (InputConfig::Disabled, OutputConfig::OpenSource) => (
            Gpio::OutputOpenDrain(
                pin.into_open_drain_output(OpenDrainConfig::Disconnect0Standard1, initial),
            ),
            Ok(()),
        ),
        (InputConfig::Floating, OutputConfig::OpenSource) => (
            Gpio::OutputOpenDrainIO(
                pin.into_open_drain_input_output(OpenDrainConfig::Disconnect0Standard1, initial),
            ),
            Ok(()),
        ),
        (InputConfig::PullDown, OutputConfig::OpenSource) => {
            let pin =
                pin.into_open_drain_input_output(OpenDrainConfig::Disconnect0Standard1, initial);
            pin_cnf(&pin).modify(|_, w| w.pull().pulldown());
            (Gpio::OutputOpenDrainIO(pin), Ok(()))
        }
        (InputConfig::PullUp, OutputConfig::OpenSource) => (wrap(pin), Err(Error::user(0))),
    }
}

fn pin_cnf(pin: &Pin<Output<OpenDrainIO>>) -> &Reg<PIN_CNF_SPEC> {
    let port = match pin.port() {
        nrf52840_hal::gpio::Port::Port0 => pac::P0::ptr(),
        nrf52840_hal::gpio::Port::Port1 => pac::P1::ptr(),
    };
    unsafe { &(*port).pin_cnf[pin.pin() as usize] }
}
