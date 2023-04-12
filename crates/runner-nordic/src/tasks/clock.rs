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
use embedded_hal::timer::Cancel;
use nrf52840_hal::pac::{TIMER0, TIMER1, TIMER2, TIMER3, TIMER4};
use nrf52840_hal::timer::{Instance, OneShot, Periodic};
use nrf52840_hal::Timer;
use {wasefire_board_api as board, wasefire_logger as logger};

impl board::timer::Api for &mut crate::tasks::Board {
    fn count(&mut self) -> usize {
        critical_section::with(|cs| self.0.borrow_ref(cs).timers.0.len())
    }

    fn arm(&mut self, i: usize, command: &board::timer::Command) -> Result<(), board::Error> {
        critical_section::with(|cs| try {
            let timers = &mut self.0.borrow_ref_mut(cs).timers;
            let timer = timers.0.get_mut(i).ok_or(board::Error::User)?;
            match command.periodic {
                true => timer.slot.set_periodic(),
                false => timer.slot.set_oneshot(),
            }
            timer.slot.start(command.duration_ms as u32 * 1000);
        })
    }

    fn disarm(&mut self, i: usize) -> Result<(), board::Error> {
        critical_section::with(|cs| try {
            let timers = &mut self.0.borrow_ref_mut(cs).timers;
            let timer = timers.0.get_mut(i).ok_or(board::Error::User)?;
            timer.slot.cancel();
        })
    }
}

pub struct Timers([ErasedTimer; 5]);

impl Timers {
    pub fn new(t0: TIMER0, t1: TIMER1, t2: TIMER2, t3: TIMER3, t4: TIMER4) -> Self {
        Timers([
            ErasedTimer::new(t0),
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
            logger::error!("Could not cancel timer.");
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
            logger::error!("Called wait but timer is not done.");
        }
    }
}
