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

use nrf52840_hal::pac::uarte0::RegisterBlock;
use nrf52840_hal::pac::{UARTE0, UARTE1};
use nrf52840_hal::prelude::OutputPin;
use nrf52840_hal::target_constants::{EASY_DMA_SIZE, SRAM_LOWER, SRAM_UPPER};
use nrf52840_hal::uarte;
use wasefire_board_api::uart::{Api, Direction, Event};
use wasefire_board_api::{Error, Id, Support};

use crate::{with_state, Board};

pub struct Uarts {
    uarte0: UARTE0,
    uarte1: UARTE1,
    states: [State; 2],
}

const READ_CAP: usize = 32;
const READ_MAX: usize = READ_CAP - 8; // 8 bytes reserved to flush the RX FIFO

struct State {
    read_ptr: *mut u8, // size READ_CAP
    read_len: usize,
    listen_read: bool,
    listen_write: bool,
}

unsafe impl Send for State {}

impl Default for State {
    fn default() -> Self {
        let read_ptr = Box::into_raw(Box::new([0; READ_CAP])) as *mut u8;
        let read_len = 0;
        State { read_ptr, read_len, listen_read: false, listen_write: false }
    }
}

pub enum Impl {}

impl Uarts {
    pub fn new(uarte0: UARTE0, mut pins: uarte::Pins, uarte1: UARTE1) -> Self {
        let mut uarts = Uarts { uarte0, uarte1, states: [State::default(), State::default()] };
        let uart = uarts.get(Id::new(0).unwrap());
        uart.regs.psel.rxd.write(|w| unsafe { w.bits(pins.rxd.psel_bits()) });
        pins.txd.set_high().unwrap();
        uart.regs.psel.txd.write(|w| unsafe { w.bits(pins.txd.psel_bits()) });
        uart.regs.psel.cts.write(|w| unsafe { w.bits(pins.cts.unwrap().psel_bits()) });
        uart.regs.psel.rts.write(|w| unsafe { w.bits(pins.rts.unwrap().psel_bits()) });
        uart.regs.config.write(|w| w.hwfc().set_bit().parity().variant(uarte::Parity::EXCLUDED));
        uart.regs.baudrate.write(|w| w.baudrate().variant(uarte::Baudrate::BAUD115200));
        uart.regs.enable.write(|w| w.enable().enabled());
        uart.start_rx();
        uarts
    }

    pub fn tick(&mut self, uart_id: Id<Impl>, mut push: impl FnMut(Event<Board>)) {
        let uart = self.get(uart_id);
        if uart.state.listen_read && uart.regs.events_rxdrdy.read().events_rxdrdy().bit_is_set() {
            uart.regs.events_rxdrdy.reset();
            push(Event { uart: uart_id, direction: Direction::Read });
        }
        if uart.state.listen_write && uart.regs.events_cts.read().events_cts().bit_is_set() {
            uart.regs.events_cts.reset();
            push(Event { uart: uart_id, direction: Direction::Write });
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
    fn start_rx(&self) {
        let ptr = unsafe { self.state.read_ptr.add(self.state.read_len) };
        let cnt = match READ_MAX.checked_sub(self.state.read_len) {
            None | Some(0) => return,
            Some(x) => x,
        };
        self.regs.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        self.regs.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(cnt as u16) });
        self.regs.tasks_startrx.write(|w| w.tasks_startrx().set_bit());
    }

    fn stop_rx(&mut self) {
        if READ_MAX <= self.state.read_len {
            // We don't start receiving in that case.
            return;
        }
        self.regs.tasks_stoprx.write(|w| w.tasks_stoprx().set_bit());
        self.wait_rxto();
        let read_len = self.wait_endrx();
        self.state.read_len += read_len;
        let ptr = unsafe { self.state.read_ptr.add(self.state.read_len) };
        let cnt = READ_CAP - self.state.read_len;
        self.regs.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        self.regs.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(cnt as u16) });
        self.regs.tasks_flushrx.write(|w| w.tasks_flushrx().set_bit());
        let flush_len = self.wait_endrx();
        self.state.read_len += flush_len.saturating_sub(read_len);
    }

    fn wait_rxto(&self) {
        // This loop is supposed to be short.
        while self.regs.events_rxto.read().events_rxto().bit_is_clear() {}
        self.regs.events_rxto.reset();
    }

    fn wait_endrx(&self) -> usize {
        // This loop is supposed to be short.
        while self.regs.events_endrx.read().events_endrx().bit_is_clear() {}
        self.regs.events_endrx.reset();
        self.regs.rxd.amount.read().amount().bits() as usize
    }

    fn stoptx(&self) {
        self.regs.tasks_stoptx.write(|w| w.tasks_stoptx().set_bit());
        while self.regs.events_txstopped.read().events_txstopped().bit_is_clear() {}
        self.regs.events_txstopped.reset();
    }
}

impl Support<usize> for Impl {
    // We deliberately advertise only the first UART for now.
    const SUPPORT: usize = 1;
}

impl Api for Impl {
    fn read(uart: Id<Self>, output: &mut [u8]) -> Result<usize, Error> {
        with_state(|state| {
            let mut uart = state.uarts.get(uart);
            uart.stop_rx();
            let len = core::cmp::min(output.len(), uart.state.read_len);
            let data = unsafe {
                core::slice::from_raw_parts_mut(uart.state.read_ptr, uart.state.read_len)
            };
            output[.. len].copy_from_slice(&data[.. len]);
            data.copy_within(len .., 0);
            uart.state.read_len = data.len() - len;
            uart.start_rx();
            Ok(len)
        })
    }

    fn write(uart: Id<Self>, input: &[u8]) -> Result<usize, Error> {
        if input.is_empty() {
            return Ok(0);
        }
        if EASY_DMA_SIZE < input.len() || !in_ram(input) {
            return Err(Error::user(0));
        }
        with_state(|state| {
            let uart = state.uarts.get(uart);
            uart.regs.txd.ptr.write(|w| unsafe { w.ptr().bits(input.as_ptr() as u32) });
            uart.regs.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(input.len() as u16) });
            uart.regs.tasks_starttx.write(|w| w.tasks_starttx().set_bit());
            while uart.regs.events_endtx.read().events_endtx().bit_is_clear()
                && uart.regs.events_ncts.read().events_ncts().bit_is_clear()
            {}
            uart.stoptx();
            uart.regs.events_endtx.reset();
            Ok(uart.regs.txd.amount.read().amount().bits() as usize)
        })
    }

    fn enable(uart: Id<Self>, direction: Direction) -> Result<(), Error> {
        with_state(|state| {
            let uart = state.uarts.get(uart);
            match direction {
                Direction::Read => {
                    uart.regs.intenset.write(|w| w.rxdrdy().set_bit());
                    uart.state.listen_read = true;
                }
                Direction::Write => {
                    uart.regs.intenset.write(|w| w.cts().set_bit());
                    uart.state.listen_write = true;
                }
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
                Direction::Write => {
                    uart.regs.intenclr.write(|w| w.cts().set_bit());
                    uart.state.listen_write = false;
                }
            }
            Ok(())
        })
    }
}

fn in_ram(xs: &[u8]) -> bool {
    let ptr = xs.as_ptr_range();
    SRAM_LOWER <= ptr.start as usize && ptr.end as usize <= SRAM_UPPER
}
