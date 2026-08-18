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

use alloc::collections::vec_deque::VecDeque;

use earlgrey::{GPIO, PINMUX_AON};
use wasefire_board_api::button::{Api, Event};
use wasefire_board_api::{Id, Support};
use wasefire_error::{Code, Error};
use wasefire_logger as log;

use crate::board::with_state;

pub struct State {
    /// The threshold to detect a touch (the moving average is below).
    threshold: u64,

    /// The history length over which the moving average is calculated.
    ///
    /// This is a trade-off between reducing noise and missing early edges after activation. The
    /// initial touch state is decided using the first complete moving average.
    history_len: usize,

    active: Option<Active>,
}

struct Active {
    history: VecDeque<u64>,
    sum: u64, // history.iter().sum()
    start: u64,
    touch: bool,
    push: bool,
}

// TODO: Use top_earlgrey_muxed_pads_ior13/ioc12 once they exist.
#[cfg(feature = "button-ior13")]
const BUTTON_PAD: u32 = 46; // IOR13
#[cfg(not(feature = "button-ior13"))]
const BUTTON_PAD: u32 = 34; // IOC12
const BUTTON_GPIO: u32 = 4;
const BUTTON_MASK: u32 = 1 << BUTTON_GPIO;

pub fn init(store: &mut wasefire_store::Store<crate::board::storage::Impl>) -> State {
    PINMUX_AON.mio_outsel(BUTTON_PAD).reset().out(3 + BUTTON_GPIO).reg.write();
    PINMUX_AON.mio_periph_insel(BUTTON_GPIO).reset().r#in(2 + BUTTON_PAD).reg.write();
    GPIO.intr_enable().modify_raw(|x| x | BUTTON_MASK);
    GPIO.ctrl_en_input_filter().modify_raw(|x| x | BUTTON_MASK);
    GPIO.masked_out_lower().reset().mask(BUTTON_MASK).data(BUTTON_MASK).reg.write();
    let config = match read(store) {
        Ok(x) => x,
        Err(e) => {
            log::warn!("failed to read captouch config: {}", e);
            DEFAULT
        }
    };
    State {
        threshold: u64::from_ne_bytes(config.threshold),
        history_len: usize::from_ne_bytes(config.history_len),
        active: None,
    }
}

pub fn interrupt() {
    with_state(|state| {
        GPIO.intr_ctrl_en_falling().modify_raw(|x| x & !BUTTON_MASK);
        GPIO.intr_state().write_raw(BUTTON_MASK);
        if let Some(active) = &mut state.button.active {
            let end = crate::time::uptime_us();
            let delta = end - active.start;
            active.history.push_back(delta);
            active.sum += delta;
            match active.history.len().cmp(&state.button.history_len) {
                core::cmp::Ordering::Less => (),
                core::cmp::Ordering::Equal => {
                    active.touch = active.average() < state.button.threshold
                }
                core::cmp::Ordering::Greater => {
                    active.sum -= active.history.pop_front().unwrap();
                    if (active.average() < state.button.threshold) != active.touch {
                        active.touch = !active.touch;
                        if active.push {
                            let button = Id::new(0).unwrap();
                            let pressed = active.touch;
                            state.events.push(Event { button, pressed }.into());
                        }
                    }
                }
            }
            active.start();
        }
    });
}

#[cfg(feature = "test-vendor")]
pub fn vendor(request: &[u8]) -> Result<alloc::boxed::Box<[u8]>, Error> {
    use alloc::boxed::Box;
    let request = request.trim_ascii_end();
    if request == b"start" {
        with_state(|state| {
            state.button.start(false)?;
            Ok(Box::default())
        })
    } else if request == b"measure" {
        with_state(|state| {
            let Some(active) = state.button.active(false)? else {
                return Err(Error::user(Code::InvalidState));
            };
            Ok(alloc::format!("{:?}\n", active.history).into_bytes().into_boxed_slice())
        })
    } else if request == b"stop" {
        with_state(|state| {
            state.button.stop(false)?;
            Ok(Box::default())
        })
    } else if request == b"dump" {
        with_state(|state| {
            let x = state.button.threshold;
            let y = state.button.history_len;
            Ok(alloc::format!("threshold: {x}\nhistory_len: {y}\n").into_bytes().into_boxed_slice())
        })
    } else if let Some(val) = request.strip_prefix(b"set_threshold ") {
        let val = try { str::from_utf8(val).ok()?.parse::<u64>().ok()? };
        let val = val.ok_or(Error::user(Code::InvalidArgument))?;
        with_state(|state| {
            if state.button.active(false)?.is_some() {
                return Err(Error::user(Code::InvalidState));
            }
            state.button.threshold = val;
            state.button.write(&mut state.storage.store)?;
            Ok(Box::default())
        })
    } else if let Some(val) = request.strip_prefix(b"set_history_len ") {
        let val = try { str::from_utf8(val).ok()?.parse::<usize>().ok()? };
        let val = val.ok_or(Error::user(Code::InvalidArgument))?;
        if val == 0 || 100 < val {
            return Err(Error::user(Code::OutOfBounds));
        }
        with_state(|state| {
            if state.button.active(false)?.is_some() {
                return Err(Error::user(Code::InvalidState));
            }
            state.button.history_len = val;
            state.button.write(&mut state.storage.store)?;
            Ok(Box::default())
        })
    } else {
        Err(Error::user(Code::InvalidArgument))
    }
}

pub enum Impl {}

impl Support<usize> for Impl {
    const SUPPORT: usize = 1;
}

impl Api for Impl {
    fn enable(button: Id<Self>) -> Result<(), Error> {
        if *button != 0 {
            return Err(Error::user(Code::OutOfBounds));
        }
        with_state(|state| state.button.start(true))
    }

    fn disable(button: Id<Self>) -> Result<(), Error> {
        if *button != 0 {
            return Err(Error::user(Code::OutOfBounds));
        }
        with_state(|state| state.button.stop(true))
    }
}

impl State {
    fn active(&mut self, push: bool) -> Result<Option<&mut Active>, Error> {
        match &mut self.active {
            None => Ok(None),
            Some(active) => {
                if active.push == push {
                    Ok(Some(active))
                } else {
                    Err(Error::user(Code::InvalidState))
                }
            }
        }
    }

    fn start(&mut self, push: bool) -> Result<(), Error> {
        if self.active(push)?.is_some() {
            return Ok(());
        }
        let active = self.active.insert(Active {
            history: VecDeque::with_capacity(self.history_len),
            sum: 0,
            start: 0,     // is set below
            touch: false, // is set when reaching the history len
            push,
        });
        active.start();
        Ok(())
    }

    fn stop(&mut self, push: bool) -> Result<(), Error> {
        if self.active(push)?.is_none() {
            return Ok(());
        }
        self.active = None;
        Ok(())
    }

    #[cfg(feature = "test-vendor")]
    fn write(
        &self, store: &mut wasefire_store::Store<crate::board::storage::Impl>,
    ) -> Result<(), Error> {
        let config = Config {
            threshold: self.threshold.to_ne_bytes(),
            history_len: self.history_len.to_ne_bytes(),
        };
        write(store, config)
    }
}

impl Active {
    fn start(&mut self) {
        GPIO.masked_oe_lower().reset().mask(BUTTON_MASK).data(BUTTON_MASK).reg.write();
        GPIO.intr_state().write_raw(BUTTON_MASK);
        GPIO.intr_ctrl_en_falling().modify_raw(|x| x | BUTTON_MASK);
        self.start = crate::time::uptime_us();
        GPIO.masked_oe_lower().reset().mask(BUTTON_MASK).data(0).reg.write();
    }

    fn average(&self) -> u64 {
        self.sum / (self.history.len() as u64)
    }
}

const KEY: usize = 0;

#[derive(Clone, Copy, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Config {
    threshold: [u8; 8],
    history_len: [u8; 4],
}

const DEFAULT: Config =
    Config { threshold: u64::to_ne_bytes(30_000), history_len: usize::to_ne_bytes(2) };

fn read(store: &mut wasefire_store::Store<crate::board::storage::Impl>) -> Result<Config, Error> {
    match store.find(KEY)? {
        None => Ok(DEFAULT),
        Some(value) if value.len() == core::mem::size_of::<Config>() => {
            Ok(*bytemuck::from_bytes(&value))
        }
        _ => Err(Error::internal(Code::InvalidState)),
    }
}

#[cfg(feature = "test-vendor")]
fn write(
    store: &mut wasefire_store::Store<crate::board::storage::Impl>, value: Config,
) -> Result<(), Error> {
    if value == DEFAULT { store.remove(KEY) } else { store.insert(KEY, bytemuck::bytes_of(&value)) }
}
