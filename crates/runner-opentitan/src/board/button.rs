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
use wasefire_board_api::button::Api;
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
    start: u64,
    touch: bool,
}

pub fn init() -> State {
    // TODO: Generate top_earlgrey_pinmux_{mio_out,outsel}. This should be MioOutIor13 and
    // GpioGpio1.
    PINMUX_AON.mio_outsel(46).reset().out(4).reg.write();
    // TODO: Generate top_earlgrey_pinmux_{peripheral_in,insel}. This should be GpioGpio1 and
    // MioOutIor13.
    PINMUX_AON.mio_periph_insel(1).reset().r#in(48).reg.write();
    GPIO.intr_enable().modify_raw(|x| x | 2);
    GPIO.ctrl_en_input_filter().modify_raw(|x| x | 2);
    GPIO.masked_out_lower().reset().mask(2).data(2).reg.write();
    State { threshold: 50000, history_len: 5, active: None }
}

pub fn interrupt() {
    with_state(|state| {
        GPIO.intr_ctrl_en_falling().modify_raw(|x| x & !2);
        GPIO.intr_state().write_raw(2);
        if let Some(active) = &mut state.button.active {
            let end = crate::time::uptime_us();
            active.history.push_back(end - active.start);
            if active.history.len() == state.button.history_len {
                active.touch = active.average() < state.button.threshold;
            }
            while state.button.history_len < active.history.len() {
                active.history.pop_front();
            }
            if (active.average() < state.button.threshold) != active.touch {
                active.touch = !active.touch;
                let button = Id::new(0).unwrap();
                let pressed = active.touch;
                state.events.push(wasefire_board_api::button::Event { button, pressed }.into());
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
            state.button.active = Some(Active {
                history: VecDeque::with_capacity(state.button.history_len),
                start: 0,     // is set below
                touch: false, // is set when reaching the history len
            });
            state.button.active.as_mut().unwrap().start();
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
        GPIO.masked_oe_lower().reset().mask(2).data(2).reg.write();
        GPIO.intr_state().write_raw(2);
        GPIO.intr_ctrl_en_falling().modify_raw(|x| x | 2);
        self.start = crate::time::uptime_us();
        GPIO.masked_oe_lower().reset().mask(2).data(0).reg.write();
    }

    fn average(&self) -> u64 {
        self.history.iter().sum::<u64>() / (self.history.len() as u64)
    }
}
