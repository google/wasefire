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
}

// TODO: Use top_earlgrey_muxed_pads_ior13 once it exists.
const BUTTON_PAD: u32 = 46; // IOR13
const BUTTON_GPIO: u32 = 1;
const BUTTON_MASK: u32 = 1 << BUTTON_GPIO;

pub fn init() -> State {
    PINMUX_AON.mio_outsel(BUTTON_PAD).reset().out(3 + BUTTON_GPIO).reg.write();
    PINMUX_AON.mio_periph_insel(BUTTON_GPIO).reset().r#in(2 + BUTTON_PAD).reg.write();
    GPIO.intr_enable().modify_raw(|x| x | BUTTON_MASK);
    GPIO.ctrl_en_input_filter().modify_raw(|x| x | BUTTON_MASK);
    GPIO.masked_out_lower().reset().mask(BUTTON_MASK).data(BUTTON_MASK).reg.write();
    // TODO: Add logic to use different values. The user should be able to set them and persist them
    // in flash. It should also be possible to guess them using some calibration logic (where the
    // user first doesn't touch the button, then touches it). It is important to make sure
    // `history_len` is positive (e.g. by using 1 if it's 0) and to not modify it while the button
    // is active.
    State { threshold: 50_000, history_len: 5, active: None }
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
                        let button = Id::new(0).unwrap();
                        let pressed = active.touch;
                        state.events.push(Event { button, pressed }.into());
                    }
                }
            }
            active.start();
        }
    });
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
        with_state(|state| {
            let active = state.button.active.insert(Active {
                history: VecDeque::with_capacity(state.button.history_len),
                sum: 0,
                start: 0,     // is set below
                touch: false, // is set when reaching the history len
            });
            active.start();
            Ok(())
        })
    }

    fn disable(button: Id<Self>) -> Result<(), Error> {
        if *button != 0 {
            return Err(Error::user(Code::OutOfBounds));
        }
        with_state(|state| Ok(state.button.active = None))
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
