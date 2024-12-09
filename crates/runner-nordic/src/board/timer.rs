// Copyright 2022 Google LLC
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

use alloc::boxed::Box;

use cortex_m::prelude::_embedded_hal_timer_CountDown;
use embedded_hal_02::timer::Cancel;
use nrf52840_hal::Timer;
use nrf52840_hal::pac::{TIMER1, TIMER2, TIMER3, TIMER4};
use nrf52840_hal::timer::{Instance, OneShot, Periodic};
use wasefire_board_api::timer::{Api, Command};
use wasefire_board_api::{Error, Id, Support};
use wasefire_logger as log;

use crate::with_state;

pub enum Impl {}

impl Support<usize> for Impl {
    const SUPPORT: usize = 4;
}

impl Api for Impl {
    fn arm(id: Id<Self>, command: &Command) -> Result<(), Error> {
        with_state(|state| {
            let timer = &mut state.timers.0[*id];
            match command.periodic {
                true => timer.slot.set_periodic(),
                false => timer.slot.set_oneshot(),
            }
            // Timers trigger after incrementing the counter, so 0 is the longest timer while 1 is
            // the shortest.
            timer.slot.start(core::cmp::max(1, command.duration_ms as u32 * 1000));
            Ok(())
        })
    }

    fn disarm(id: Id<Self>) -> Result<(), Error> {
        with_state(|state| {
            state.timers.0[*id].slot.cancel();
            Ok(())
        })
    }
}

pub struct Timers([ErasedTimer; <Impl as Support<usize>>::SUPPORT]);

impl Timers {
    pub fn new(t1: TIMER1, t2: TIMER2, t3: TIMER3, t4: TIMER4) -> Self {
        Timers([
            ErasedTimer::new(t1),
            ErasedTimer::new(t2),
            ErasedTimer::new(t3),
            ErasedTimer::new(t4),
        ])
    }

    pub fn tick(&mut self, index: usize) {
        self.0[index].tick();
    }
}

struct ErasedTimer {
    slot: Box<dyn ErasedSlot + Send>,
}

impl ErasedTimer {
    fn new<T: Instance + Send + 'static>(x: T) -> Self {
        x.enable_interrupt();
        ErasedTimer { slot: Box::new(Slot::new(x)) }
    }

    fn tick(&mut self) {
        self.slot.wait();
    }
}

enum Slot<T: Instance> {
    Invalid,
    OneShot(Timer<T, OneShot>),
    Periodic(Timer<T, Periodic>),
}

impl<T: Instance> Slot<T> {
    fn new(x: T) -> Self {
        Slot::OneShot(Timer::new(x))
    }
}

trait ErasedSlot {
    fn cancel(&mut self);
    fn set_oneshot(&mut self);
    fn set_periodic(&mut self);
    fn start(&mut self, cycles: u32);
    fn wait(&mut self);
}

impl<T: Instance + Send> ErasedSlot for Slot<T> {
    fn cancel(&mut self) {
        let result = match self {
            Slot::Invalid => unreachable!(),
            Slot::OneShot(x) => x.cancel(),
            Slot::Periodic(x) => x.cancel(),
        };
        if result.is_err() {
            log::error!("Could not cancel timer.");
        }
    }

    fn set_oneshot(&mut self) {
        *self = match core::mem::replace(self, Slot::Invalid) {
            Slot::Periodic(x) => Slot::OneShot(x.into_oneshot()),
            x => x,
        };
    }

    fn set_periodic(&mut self) {
        *self = match core::mem::replace(self, Slot::Invalid) {
            Slot::OneShot(x) => Slot::Periodic(x.into_periodic()),
            x => x,
        };
    }

    fn start(&mut self, cycles: u32) {
        match self {
            Slot::Invalid => unreachable!(),
            Slot::OneShot(x) => x.start(cycles),
            Slot::Periodic(x) => x.start(cycles),
        }
    }

    fn wait(&mut self) {
        let done = match self {
            Slot::Invalid => unreachable!(),
            Slot::OneShot(x) => x.wait().is_ok(),
            Slot::Periodic(x) => x.wait().is_ok(),
        };
        if !done {
            log::error!("Called wait but timer is not done.");
        }
    }
}
