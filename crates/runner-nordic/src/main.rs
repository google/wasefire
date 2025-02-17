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

#![no_std]
#![no_main]
#![feature(never_type)]
#![feature(ptr_metadata)]
#![feature(try_blocks)]

extern crate alloc;

mod allocator;
mod board;
mod storage;
#[cfg(feature = "debug")]
mod systick;

use alloc::vec::Vec;
use core::cell::RefCell;
use core::mem::MaybeUninit;

use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use critical_section::Mutex;
#[cfg(feature = "debug")]
use defmt_rtt as _;
use embedded_hal::digital::InputPin;
#[cfg(feature = "aes128-ccm")]
use nrf52840_hal::ccm::{Ccm, DataRate};
use nrf52840_hal::clocks::{self, ExternalOscillator, Internal, LfOscStopped};
use nrf52840_hal::gpio::{self, Level, Output, Pin, PushPull};
use nrf52840_hal::gpiote::Gpiote;
use nrf52840_hal::pac::{FICR, Interrupt, interrupt};
use nrf52840_hal::rng::Rng;
use nrf52840_hal::usbd::{UsbPeripheral, Usbd};
#[cfg(feature = "release")]
use panic_abort as _;
#[cfg(feature = "debug")]
use panic_probe as _;
#[cfg(feature = "radio-ble")]
use rubble::link::MIN_PDU_BUF;
#[cfg(feature = "radio-ble")]
use rubble_nrf5x::radio::BleRadio;
use usb_device::class::UsbClass;
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid};
#[cfg(feature = "usb-ctap")]
use usbd_hid::descriptor::{CtapReport, SerializedDescriptor};
#[cfg(feature = "usb-ctap")]
use usbd_hid::hid_class::HIDClass;
#[cfg(feature = "usb-serial")]
use usbd_serial::SerialPort;
#[cfg(feature = "usb-ctap")]
use wasefire_board_api::usb::ctap::Ctap;
#[cfg(feature = "usb-serial")]
use wasefire_board_api::usb::serial::Serial;
use wasefire_board_api::{Id, Support};
#[cfg(feature = "wasm")]
use wasefire_interpreter as _;
use wasefire_logger as log;
use wasefire_one_of::exactly_one_of;
use wasefire_scheduler::Scheduler;

use crate::board::button::{Button, channel};
#[cfg(feature = "gpio")]
use crate::board::gpio::Gpio;
#[cfg(feature = "radio-ble")]
use crate::board::radio::ble::Ble;
use crate::board::timer::Timers;
#[cfg(feature = "uart")]
use crate::board::uart::Uarts;
use crate::board::usb::Usb;
use crate::board::{Events, button, led};
use crate::storage::Storage;

exactly_one_of!["debug", "release"];
exactly_one_of!["native", "wasm"];

#[cfg(feature = "debug")]
#[defmt::panic_handler]
fn panic() -> ! {
    panic_probe::hard_fault();
}

type Clocks = clocks::Clocks<ExternalOscillator, Internal, LfOscStopped>;

struct State {
    events: Events,
    ficr: FICR,
    buttons: [Button; <button::Impl as Support<usize>>::SUPPORT],
    gpiote: Gpiote,
    protocol: wasefire_protocol_usb::Rpc<'static, Usb>,
    #[cfg(feature = "usb-ctap")]
    ctap: Ctap<'static, Usb>,
    #[cfg(feature = "usb-serial")]
    serial: Serial<'static, Usb>,
    timers: Timers,
    #[cfg(feature = "radio-ble")]
    ble: Ble,
    #[cfg(feature = "aes128-ccm")]
    ccm: Ccm,
    #[cfg(feature = "gpio")]
    gpios: [Gpio; <board::gpio::Impl as Support<usize>>::SUPPORT],
    leds: [Pin<Output<PushPull>>; <led::Impl as Support<usize>>::SUPPORT],
    rng: Rng,
    storage: Option<Storage>,
    #[cfg(feature = "uart")]
    uarts: Uarts,
    usb_dev: UsbDevice<'static, Usb>,
}

pub enum Board {}

static STATE: Mutex<RefCell<Option<State>>> = Mutex::new(RefCell::new(None));

fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    critical_section::with(|cs| f(STATE.borrow_ref_mut(cs).as_mut().unwrap()))
}

#[entry]
fn main() -> ! {
    static mut CLOCKS: MaybeUninit<Clocks> = MaybeUninit::uninit();
    static mut USB_BUS: MaybeUninit<UsbBusAllocator<Usb>> = MaybeUninit::uninit();

    #[cfg(feature = "debug")]
    let c = nrf52840_hal::pac::CorePeripherals::take().unwrap();
    #[cfg(feature = "debug")]
    systick::init(c.SYST);
    allocator::init();
    log::debug!("Runner starts.");
    let p = nrf52840_hal::pac::Peripherals::take().unwrap();
    let ficr = p.FICR;
    let port0 = gpio::p0::Parts::new(p.P0);
    #[cfg(feature = "gpio")]
    let port1 = gpio::p1::Parts::new(p.P1);
    let buttons = [
        Button::new(port0.p0_11.into_pullup_input().degrade()),
        Button::new(port0.p0_12.into_pullup_input().degrade()),
        Button::new(port0.p0_24.into_pullup_input().degrade()),
        Button::new(port0.p0_25.into_pullup_input().degrade()),
    ];
    let leds = [
        port0.p0_13.into_push_pull_output(Level::High).degrade(),
        port0.p0_14.into_push_pull_output(Level::High).degrade(),
        port0.p0_15.into_push_pull_output(Level::High).degrade(),
        port0.p0_16.into_push_pull_output(Level::High).degrade(),
    ];
    #[cfg(feature = "gpio")]
    let gpios = [
        Gpio::new(port1.p1_01.degrade(), 1, 1),
        Gpio::new(port1.p1_02.degrade(), 1, 2),
        Gpio::new(port1.p1_03.degrade(), 1, 3),
        Gpio::new(port1.p1_04.degrade(), 1, 4),
        Gpio::new(port0.p0_02.degrade(), 0, 2),
        Gpio::new(port0.p0_27.degrade(), 0, 27),
        Gpio::new(port0.p0_30.degrade(), 0, 30),
        Gpio::new(port0.p0_31.degrade(), 0, 31),
    ];
    let timers = Timers::new(p.TIMER1, p.TIMER2, p.TIMER3, p.TIMER4);
    let gpiote = Gpiote::new(p.GPIOTE);
    // We enable all USB interrupts except STARTED and EPDATA which are feedback loops.
    p.USBD.inten.write(|w| unsafe { w.bits(0x00fffffd) });
    let clocks = CLOCKS.write(clocks::Clocks::new(p.CLOCK).enable_ext_hfosc());
    let usb_bus = UsbBusAllocator::new(Usbd::new(UsbPeripheral::new(p.USBD, clocks)));
    let usb_bus = USB_BUS.write(usb_bus);
    let protocol = wasefire_protocol_usb::Rpc::new(usb_bus);
    #[cfg(feature = "usb-ctap")]
    let ctap = Ctap::new(HIDClass::new(usb_bus, CtapReport::desc(), 255));
    #[cfg(feature = "usb-serial")]
    let serial = Serial::new(SerialPort::new(usb_bus));
    let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::new(usb_device::LangID::EN).product("Wasefire")])
        .unwrap()
        .build();
    #[cfg(feature = "radio-ble")]
    let ble = {
        use alloc::boxed::Box;
        // TX buffer is mandatory even when we only listen.
        let ble_tx = Box::leak(Box::new([0; MIN_PDU_BUF]));
        let ble_rx = Box::leak(Box::new([0; MIN_PDU_BUF]));
        let radio = BleRadio::new(p.RADIO, &ficr, ble_tx, ble_rx);
        Ble::new(radio, p.TIMER0)
    };
    let rng = Rng::new(p.RNG);
    #[cfg(feature = "aes128-ccm")]
    let ccm = Ccm::init(p.CCM, p.AAR, DataRate::_1Mbit);
    storage::init(p.NVMC);
    let storage = Some(Storage::new_store());
    crate::board::platform::update::init(Storage::new_other());
    crate::board::applet::init(Storage::new_applet());
    #[cfg(feature = "uart")]
    let uarts = {
        let uart_rx = port0.p0_28.into_floating_input().degrade();
        let uart_tx = port0.p0_29.into_push_pull_output(gpio::Level::High).degrade();
        Uarts::new(p.UARTE0, uart_rx, uart_tx, p.UARTE1)
    };
    let events = Events::default();
    let state = State {
        events,
        ficr,
        buttons,
        gpiote,
        protocol,
        #[cfg(feature = "usb-ctap")]
        ctap,
        #[cfg(feature = "usb-serial")]
        serial,
        timers,
        #[cfg(feature = "radio-ble")]
        ble,
        #[cfg(feature = "aes128-ccm")]
        ccm,
        #[cfg(feature = "gpio")]
        gpios,
        leds,
        rng,
        storage,
        #[cfg(feature = "uart")]
        uarts,
        usb_dev,
    };
    // We first set the board and then enable interrupts so that interrupts may assume the board is
    // always present.
    critical_section::with(|cs| STATE.replace(cs, Some(state)));
    for &interrupt in INTERRUPTS {
        unsafe { NVIC::unmask(interrupt) };
    }
    log::debug!("Runner is initialized.");
    Scheduler::<Board>::run();
}

macro_rules! interrupts {
    ($($(#[$meta:meta])* $name:ident = $func:ident($($arg:expr),*$(,)?)),*$(,)?) => {
        const INTERRUPTS: &[Interrupt] = &[$($(#[$meta])* Interrupt::$name),*];
        $(
            $(#[$meta])*
            #[interrupt]
            fn $name() {
                $func($($arg),*);
            }
        )*
    };
}

interrupts! {
    GPIOTE = gpiote(),
    #[cfg(feature = "radio-ble")]
    RADIO = radio(),
    #[cfg(feature = "radio-ble")]
    TIMER0 = radio_timer(),
    TIMER1 = timer(0),
    TIMER2 = timer(1),
    TIMER3 = timer(2),
    TIMER4 = timer(3),
    #[cfg(feature = "uart")]
    UARTE0_UART0 = uarte(0),
    #[cfg(feature = "uart")]
    UARTE1 = uarte(1),
    USBD = usbd(),
}

fn gpiote() {
    with_state(|state| {
        for (i, button) in state.buttons.iter_mut().enumerate() {
            let id = Id::new(i).unwrap();
            if channel(&state.gpiote, id).is_event_triggered() {
                let pressed = button.pin.is_low().unwrap();
                state.events.push(wasefire_board_api::button::Event { button: id, pressed }.into());
            }
        }
        state.gpiote.reset_events();
    });
}

#[cfg(feature = "radio-ble")]
fn radio() {
    with_state(|state| state.ble.tick(|event| state.events.push(event.into())))
}

#[cfg(feature = "radio-ble")]
fn radio_timer() {
    with_state(|state| state.ble.tick_timer())
}

fn timer(timer: usize) {
    let timer = Id::new(timer).unwrap();
    with_state(|state| {
        state.events.push(wasefire_board_api::timer::Event { timer }.into());
        state.timers.tick(*timer);
    })
}

#[cfg(feature = "uart")]
fn uarte(uarte: usize) {
    let uart = Id::new(uarte).unwrap();
    with_state(|state| {
        state.uarts.tick(uart, |event| state.events.push(event.into()));
    })
}

fn usbd() {
    with_state(|state| {
        let mut classes = Vec::<&mut dyn UsbClass<_>>::new();
        classes.push(&mut state.protocol);
        #[cfg(feature = "usb-ctap")]
        classes.push(state.ctap.class());
        #[cfg(feature = "usb-serial")]
        classes.push(state.serial.port());
        #[cfg_attr(not(feature = "usb-serial"), allow(unused_variables))]
        let polled = state.usb_dev.poll(&mut classes);
        state.protocol.tick(|event| state.events.push(event.into()));
        #[cfg(feature = "usb-ctap")]
        state.ctap.tick(|event| state.events.push(event.into()));
        #[cfg(feature = "usb-serial")]
        state.serial.tick(polled, |event| state.events.push(event.into()));
    });
}
