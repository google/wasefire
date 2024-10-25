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

use alloc::boxed::Box;

use embedded_hal::digital::OutputPin;
use nrf52840_hal::pac::uarte0::{errorsrc, RegisterBlock};
use nrf52840_hal::pac::{UARTE0, UARTE1};
use nrf52840_hal::target_constants::{EASY_DMA_SIZE, SRAM_LOWER, SRAM_UPPER};
use nrf52840_hal::{gpio, uarte};
use wasefire_board_api::uart::{Api, Direction, Event};
use wasefire_board_api::{Error, Id, Support};
use wasefire_error::Code;
use wasefire_logger as log;

use crate::{with_state, Board};

pub struct Uarts {
    uarte0: UARTE0,
    uarte1: UARTE1,
    states: [State; 2],
}

const BUFFER_SIZE: usize = 8 * BUSY_SIZE;
const BUSY_SIZE: usize = 128; // must divide BUFFER_SIZE and be smaller than EASY_DMA_SIZE

struct State {
    // We use a cyclic buffer of size BUFFER_SIZE for reading, and partition it in 3 parts:
    //
    // - The uninitialized part (possibly empty) contains uninitialized data. Bytes in this part
    //   are never read nor written until they move to another part.
    //
    // - The ready part (possibly empty) contains initialized data that has been received but not
    //   yet read (thus ready to be read). Bytes in this part may be read but not written until
    //   they move to another part.
    //
    // - The busy part contains the data used or reserved for EasyDMA. Bytes in this part must not
    //   be read nor written until they move to another part. The busy part is BUSY_SIZE aligned
    //   and contains at least BUSY_SIZE bytes. It may contain an additional BUSY_SIZE bytes if the
    //   RXD.PTR field points right after the first BUSY_SIZE bytes (otherwise it must point to the
    //   first BUSY_SIZE bytes).
    //
    // This could be graphically be represented as (all values are modulo BUFFER_SIZE):
    //
    //     ... uninit  |   ready   |   busy   |  ... (cycles back to uninit)
    //                 ^read_beg   ^read_end
    //
    // We have the following invariants:
    // - `read_ptr` is a valid pointer of size BUFFER_SIZE
    // - `read_beg` is less than BUFFER_SIZE
    // - `read_end` is less than BUFFER_SIZE and divides BUSY_SIZE
    read_ptr: *mut u8,
    read_beg: usize,
    read_end: usize,

    running: bool,
    listen_read: bool,
}

unsafe impl Send for State {}

impl Default for State {
    fn default() -> Self {
        State {
            read_ptr: Box::into_raw(Box::new([0; BUFFER_SIZE])) as *mut u8,
            read_beg: 0,
            read_end: 0,
            running: false,
            listen_read: false,
        }
    }
}

pub enum Impl {}

impl Uarts {
    // We only support UARTE0 without control flow.
    pub fn new(
        uarte0: UARTE0, rx: gpio::Pin<gpio::Input<gpio::Floating>>,
        tx: gpio::Pin<gpio::Output<gpio::PushPull>>, uarte1: UARTE1,
    ) -> Self {
        let mut uarts = Uarts { uarte0, uarte1, states: [State::default(), State::default()] };
        let Uart { regs, .. } = uarts.get(Id::new(0).unwrap());
        regs.psel.rxd.write(|w| unsafe { w.bits(rx.psel_bits()) });
        regs.psel.txd.write(|w| unsafe { w.bits(tx.psel_bits()) });
        regs.config.reset();
        regs.baudrate.write(|w| w.baudrate().variant(uarte::Baudrate::BAUD115200));
        uarts
    }

    pub fn tick(&mut self, uart_id: Id<Impl>, push: impl FnMut(Event<Board>)) {
        let Uart { regs, state: s } = self.get(uart_id);
        if regs.events_error.read().events_error().bit_is_set() {
            regs.events_error.reset();
            regs.errorsrc.modify(|r, w| {
                log::warn!("{}", PrettyError(r));
                unsafe { w.bits(r.bits()) };
                w
            });
        }
        if regs.events_rxstarted.read().events_rxstarted().bit_is_set() {
            regs.events_rxstarted.reset();
            s.read_end = (regs.rxd.ptr.read().ptr().bits() - s.read_ptr as u32) as usize;
            let next_end = (s.read_end + BUSY_SIZE) % BUFFER_SIZE;
            if BUSY_SIZE <= dist(next_end, s.read_beg) {
                let next_ptr = s.read_ptr as u32 + next_end as u32;
                regs.rxd.ptr.write(|w| unsafe { w.ptr().bits(next_ptr) });
            }
        }
        if s.listen_read {
            Self::tick_read(uart_id, regs, push);
        }
    }

    fn tick_read(uart: Id<Impl>, regs: &RegisterBlock, mut push: impl FnMut(Event<Board>)) {
        if regs.events_rxdrdy.read().events_rxdrdy().bit_is_set() {
            regs.events_rxdrdy.reset();
            regs.intenclr.write(|w| w.rxdrdy().set_bit());
            push(Event { uart, direction: Direction::Read });
        }
    }

    fn get(&mut self, uart: Id<Impl>) -> Uart {
        let regs: &RegisterBlock = match *uart {
            0 => &self.uarte0,
            1 => &self.uarte1,
            _ => unreachable!(),
        };
        let state = &mut self.states[*uart];
        Uart { regs, state }
    }
}

struct Uart<'a> {
    regs: &'a RegisterBlock,
    state: &'a mut State,
}

impl<'a> Uart<'a> {
    fn stoptx(regs: &RegisterBlock) {
        regs.tasks_stoptx.write(|w| w.tasks_stoptx().set_bit());
        while regs.events_txstopped.read().events_txstopped().bit_is_clear() {}
        regs.events_txstopped.reset();
    }

    fn stoprx(regs: &RegisterBlock) {
        regs.tasks_stoprx.write(|w| w.tasks_stoprx().set_bit());
        while regs.events_rxto.read().events_rxto().bit_is_clear() {}
        regs.events_rxto.reset();
    }
}

impl State {
    /// Returns whether the ready part is empty.
    fn is_empty(&self) -> bool {
        self.read_end == self.read_beg
    }

    /// Copies the longest continuous prefix of the ready part to the output.
    ///
    /// Updates both the ready part and the output.
    fn copy(&mut self, output: &mut &mut [u8]) {
        let end = if self.read_end < self.read_beg { BUFFER_SIZE } else { self.read_end };
        let len = core::cmp::min(end - self.read_beg, output.len());
        let ptr = unsafe { self.read_ptr.add(self.read_beg) };
        output[.. len].copy_from_slice(unsafe { core::slice::from_raw_parts(ptr, len) });
        *output = &mut core::mem::take(output)[len ..];
        self.read_beg = (self.read_beg + len) % BUFFER_SIZE;
    }
}

impl Support<usize> for Impl {
    // We deliberately advertise only the first UART for now.
    const SUPPORT: usize = 1;
}

impl Api for Impl {
    fn set_baudrate(uart: Id<Self>, baudrate: usize) -> Result<(), Error> {
        let baudrate = match baudrate {
            9600 => uarte::Baudrate::BAUD9600,
            31250 => uarte::Baudrate::BAUD31250,
            38400 => uarte::Baudrate::BAUD38400,
            115200 => uarte::Baudrate::BAUD115200,
            250000 => uarte::Baudrate::BAUD250000,
            1000000 => uarte::Baudrate::BAUD1M,
            _ => return Err(Error::internal(Code::NotImplemented)),
        };
        with_state(|state| {
            let Uart { regs, state } = state.uarts.get(uart);
            Error::user(Code::InvalidState).check(!state.running)?;
            regs.baudrate.write(|w| w.baudrate().variant(baudrate));
            Ok(())
        })
    }

    fn start(uart: Id<Self>) -> Result<(), Error> {
        with_state(|state| {
            let Uart { regs, state } = state.uarts.get(uart);
            Error::user(Code::InvalidState).check(!state.running)?;
            state.running = true;
            set_high(regs.psel.txd.read().bits());
            regs.enable.write(|w| w.enable().enabled());
            regs.shorts.write(|w| w.endrx_startrx().set_bit());
            regs.intenset.write(|w| w.error().set_bit().rxstarted().set_bit());
            regs.rxd.ptr.write(|w| unsafe { w.ptr().bits(state.read_ptr as u32) });
            regs.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(BUSY_SIZE as u16) });
            regs.tasks_startrx.write(|w| w.tasks_startrx().set_bit());
            Ok(())
        })
    }

    fn stop(uart: Id<Self>) -> Result<(), Error> {
        with_state(|state| {
            let Uart { regs, state } = state.uarts.get(uart);
            Error::user(Code::InvalidState).check(state.running)?;
            state.running = false;
            regs.shorts.write(|w| w.endrx_startrx().clear_bit());
            Uart::stoptx(regs);
            Uart::stoprx(regs);
            state.read_beg = 0;
            state.read_end = 0;
            regs.enable.write(|w| w.enable().disabled());
            Ok(())
        })
    }

    fn read(uart: Id<Self>, mut output: &mut [u8]) -> Result<usize, Error> {
        with_state(|state| {
            let Uart { state: s, .. } = state.uarts.get(uart);
            Error::user(Code::InvalidState).check(s.running)?;
            let initial_len = output.len();
            s.copy(&mut output);
            if !output.is_empty() && !s.is_empty() {
                // We neither filled output nor consumed all the ready part. This can only happen if
                // the ready part was discontinuous.
                assert_eq!(s.read_beg, 0);
                s.copy(&mut output);
            }
            Ok(initial_len - output.len())
        })
    }

    fn write(uart: Id<Self>, input: &[u8]) -> Result<usize, Error> {
        if input.is_empty() {
            return Ok(0);
        }
        if EASY_DMA_SIZE < input.len() || !in_ram(input) {
            return Err(Error::user(Code::InvalidArgument));
        }
        with_state::<Result<_, Error>>(|state| {
            let Uart { regs, state } = state.uarts.get(uart);
            Error::user(Code::InvalidState).check(state.running)?;
            regs.txd.ptr.write(|w| unsafe { w.ptr().bits(input.as_ptr() as u32) });
            regs.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(input.len() as u16) });
            regs.tasks_starttx.write(|w| w.tasks_starttx().set_bit());
            Ok(())
        })?;
        loop {
            let get_amount = |state: &mut crate::State| {
                let Uart { regs, .. } = state.uarts.get(uart);
                if regs.events_endtx.read().events_endtx().bit_is_set() {
                    regs.events_endtx.reset();
                    Some(regs.txd.amount.read().amount().bits() as usize)
                } else {
                    None
                }
            };
            match with_state(get_amount) {
                Some(x) => break Ok(x),
                None => continue,
            }
        }
    }

    fn enable(uart: Id<Self>, direction: Direction) -> Result<(), Error> {
        with_state(|state| {
            let Uart { regs, state } = state.uarts.get(uart);
            match direction {
                Direction::Read => {
                    regs.intenset.write(|w| w.rxdrdy().set_bit());
                    state.listen_read = true;
                }
                Direction::Write => (),
            }
            Ok(())
        })
    }

    fn disable(uart: Id<Self>, direction: Direction) -> Result<(), Error> {
        with_state(|state| {
            let uart = state.uarts.get(uart);
            match direction {
                Direction::Read => {
                    uart.regs.intenclr.write(|w| w.rxdrdy().set_bit());
                    uart.state.listen_read = false;
                }
                Direction::Write => (),
            }
            Ok(())
        })
    }
}

fn in_ram(xs: &[u8]) -> bool {
    let ptr = xs.as_ptr_range();
    SRAM_LOWER <= ptr.start as usize && ptr.end as usize <= SRAM_UPPER
}

fn set_high(psel_bits: u32) {
    use nrf52840_hal::gpio::{Output, Pin, PushPull};
    unsafe { Pin::<Output<PushPull>>::from_psel_bits(psel_bits) }.set_high().unwrap();
}

#[allow(dead_code)]
struct PrettyError<'a>(&'a errorsrc::R);

#[cfg(feature = "debug")]
impl<'a> defmt::Format for PrettyError<'a> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "UART error:");
        if self.0.overrun().is_present() {
            defmt::write!(fmt, " OVERRUN");
        }
        if self.0.parity().is_present() {
            defmt::write!(fmt, " PARITY");
        }
        if self.0.framing().is_present() {
            defmt::write!(fmt, " FRAMING");
        }
        if self.0.break_().is_present() {
            defmt::write!(fmt, " BREAK");
        }
    }
}

/// Returns the distance between 2 ordered points in the cyclic buffer.
///
/// If both points are equal, the distance is zero.
fn dist(a: usize, b: usize) -> usize {
    match b.checked_sub(a) {
        Some(x) => x,
        None => b + BUFFER_SIZE - a,
    }
}
