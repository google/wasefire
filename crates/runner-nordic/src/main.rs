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
#![feature(try_blocks)]

extern crate alloc;

mod allocator;
mod storage;
#[cfg(feature = "debug")]
mod systick;
mod tasks;

use alloc::collections::VecDeque;
use core::cell::RefCell;
use core::mem::MaybeUninit;

use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use critical_section::Mutex;
#[cfg(feature = "debug")]
use defmt_rtt as _;
use nrf52840_hal::ccm::{Ccm, DataRate};
use nrf52840_hal::clocks::{self, ExternalOscillator, Internal, LfOscStopped};
use nrf52840_hal::gpio::{Level, Output, Pin, PushPull};
use nrf52840_hal::gpiote::Gpiote;
use nrf52840_hal::pac::{interrupt, Interrupt, TIMER0};
use nrf52840_hal::prelude::InputPin;
use nrf52840_hal::rng::Rng;
use nrf52840_hal::usbd::{UsbPeripheral, Usbd};
use nrf52840_hal::{gpio, uarte};
#[cfg(feature = "release")]
use panic_abort as _;
#[cfg(feature = "debug")]
use panic_probe as _;
use rubble::beacon::{BeaconScanner, ScanCallback};
use rubble::bytes::{ByteWriter, ToBytes};
use rubble::link::ad_structure::AdStructure;
use rubble::link::filter::AllowAll;
use rubble::link::{DeviceAddress, Metadata, MIN_PDU_BUF};
use rubble::time::Timer;
use rubble_nrf5x::radio::{BleRadio, PacketBuffer};
use rubble_nrf5x::timer::BleTimer;
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use wasefire_board_api::usb::serial::Serial;
use wasefire_board_api::{Id, Support};
use wasefire_scheduler::Scheduler;
use {wasefire_board_api as board, wasefire_logger as log};

use crate::storage::Storage;
use crate::tasks::button::{self, channel, Button};
use crate::tasks::clock::Timers;
use crate::tasks::uart::Uarts;
use crate::tasks::usb::Usb;
use crate::tasks::{led, Events};

#[cfg(feature = "debug")]
#[defmt::panic_handler]
fn panic() -> ! {
    panic_probe::hard_fault();
}

type Clocks = clocks::Clocks<ExternalOscillator, Internal, LfOscStopped>;

struct State {
    events: Events,
    buttons: [Button; <button::Impl as Support<usize>>::SUPPORT],
    gpiote: Gpiote,
    serial: Serial<'static, Usb>,
    timers: Timers,
    ccm: Ccm,
    leds: [Pin<Output<PushPull>>; <led::Impl as Support<usize>>::SUPPORT],
    ble_radio: BleRadio,
    ble_scanner: BeaconScanner<BleAdvScanCallback, AllowAll>,
    ble_timer: BleTimer<TIMER0>,
    ble_packet_queue: VecDeque<BlePacket>,
    rng: Rng,
    storage: Option<Storage>,
    uarts: Uarts,
    usb_dev: UsbDevice<'static, Usb>,
}

pub enum Board {}

static STATE: Mutex<RefCell<Option<State>>> = Mutex::new(RefCell::new(None));
static BLE_PACKET: Mutex<RefCell<Option<BlePacket>>> = Mutex::new(RefCell::new(None));

fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    critical_section::with(|cs| f(STATE.borrow_ref_mut(cs).as_mut().unwrap()))
}

#[entry]
fn main() -> ! {
    static mut CLOCKS: MaybeUninit<Clocks> = MaybeUninit::uninit();
    static mut USB_BUS: MaybeUninit<UsbBusAllocator<Usb>> = MaybeUninit::uninit();
    // TX buffer is mandatory even when we only listen.
    static mut BLE_TX: MaybeUninit<PacketBuffer> = MaybeUninit::uninit();
    static mut BLE_RX: MaybeUninit<PacketBuffer> = MaybeUninit::uninit();

    #[cfg(feature = "debug")]
    let c = nrf52840_hal::pac::CorePeripherals::take().unwrap();
    #[cfg(feature = "debug")]
    systick::init(c.SYST);
    allocator::init();
    log::debug!("Runner starts.");
    let p = nrf52840_hal::pac::Peripherals::take().unwrap();
    let port0 = gpio::p0::Parts::new(p.P0);
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
    let timers = Timers::new(p.TIMER1, p.TIMER2, p.TIMER3, p.TIMER4);
    let gpiote = Gpiote::new(p.GPIOTE);
    // We enable all USB interrupts except STARTED and EPDATA which are feedback loops.
    p.USBD.inten.write(|w| unsafe { w.bits(0x00fffffd) });
    let clocks = CLOCKS.write(clocks::Clocks::new(p.CLOCK).enable_ext_hfosc());
    let usb_bus = UsbBusAllocator::new(Usbd::new(UsbPeripheral::new(p.USBD, clocks)));
    let usb_bus = USB_BUS.write(usb_bus);
    let serial = Serial::new(SerialPort::new(usb_bus));
    let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::new(usb_device::LangID::EN).product("Serial port")])
        .unwrap()
        .device_class(USB_CLASS_CDC)
        .build();
    let ble_radio = BleRadio::new(
        p.RADIO,
        &p.FICR,
        BLE_TX.write([0; MIN_PDU_BUF]),
        BLE_RX.write([0; MIN_PDU_BUF]),
    );
    let ble_timer = BleTimer::init(p.TIMER0);
    let ble_scanner = BeaconScanner::new(BleAdvScanCallback);
    let ble_packet_queue = VecDeque::<BlePacket>::new();
    let rng = Rng::new(p.RNG);
    let ccm = Ccm::init(p.CCM, p.AAR, DataRate::_1Mbit);
    storage::init(p.NVMC);
    let storage = Some(Storage::new_store());
    crate::tasks::platform::update::init(Storage::new_other());
    let pins = uarte::Pins {
        txd: port0.p0_06.into_push_pull_output(gpio::Level::High).degrade(),
        rxd: port0.p0_08.into_floating_input().degrade(),
        cts: Some(port0.p0_07.into_floating_input().degrade()),
        rts: Some(port0.p0_05.into_push_pull_output(gpio::Level::High).degrade()),
    };
    let uarts = Uarts::new(p.UARTE0, pins, p.UARTE1);
    let events = Events::default();
    let state = State {
        events,
        buttons,
        gpiote,
        serial,
        timers,
        ccm,
        leds,
        ble_radio,
        ble_scanner,
        ble_timer,
        ble_packet_queue,
        rng,
        storage,
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
    #[cfg(feature = "wasm")]
    const WASM: &[u8] = include_bytes!("../../../target/wasefire/applet.wasm");
    #[cfg(feature = "wasm")]
    Scheduler::<Board>::run(WASM);
    #[cfg(feature = "native")]
    Scheduler::<Board>::run();
}

pub struct RadioMetadata {
    ticks: u32,
    freq: u16,
    rssi: i8,
    pdu_type: u8,
}

impl RadioMetadata {
    pub fn len(&self) -> usize {
        core::mem::size_of::<u32>()
            + core::mem::size_of::<u16>()
            + core::mem::size_of::<i8>()
            + core::mem::size_of::<u8>()
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

impl From<Metadata> for RadioMetadata {
    fn from(value: Metadata) -> Self {
        RadioMetadata {
            ticks: value.timestamp.unwrap().ticks(),
            freq: match value.channel {
                ch @ 0 ..= 10 => 2404 + 2 * (ch as u16),
                ch @ 11 ..= 36 => 2404 + 2 * (ch as u16 + 1),
                37 => 2402,
                38 => 2426,
                39 => 2480,
                _ => 0,
            },
            rssi: value.rssi.unwrap(),
            pdu_type: u8::from(value.pdu_type.unwrap()),
        }
    }
}

pub struct BlePacket {
    addr: [u8; 6],
    metadata: RadioMetadata,
    data: alloc::vec::Vec<u8>,
}

impl BlePacket {
    pub fn len(&self) -> usize {
        self.addr.len() + self.metadata.len() + self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

pub struct BleAdvScanCallback;

impl ScanCallback for BleAdvScanCallback {
    fn beacon<'a, I>(&mut self, addr: DeviceAddress, data: I, metadata: Metadata)
    where I: Iterator<Item = AdStructure<'a>> {
        let mut buf: [u8; MIN_PDU_BUF] = [0; MIN_PDU_BUF];
        let mut writer = ByteWriter::new(&mut buf);
        for p in data {
            assert!(p.to_bytes(&mut writer).is_ok());
        }
        let len = MIN_PDU_BUF - writer.space_left();
        let packet = BlePacket {
            addr: *addr.raw(),
            metadata: metadata.clone().into(),
            data: buf[.. len].to_vec(),
        };
        assert!(critical_section::with(|cs| BLE_PACKET.replace(cs, Some(packet))).is_none());
    }
}

macro_rules! interrupts {
    ($($name:ident = $func:ident($($arg:expr),*$(,)?)),*$(,)?) => {
        const INTERRUPTS: &[Interrupt] = &[$(Interrupt::$name),*];
        $(
            #[interrupt]
            fn $name() {
                $func($($arg),*);
            }
        )*
    };
}

interrupts! {
    GPIOTE = gpiote(),
    RADIO = radio(),
    TIMER0 = radio_timer(),
    TIMER1 = timer(1),
    TIMER2 = timer(2),
    TIMER3 = timer(3),
    TIMER4 = timer(4),
    UARTE0_UART0 = uarte(0),
    UARTE1 = uarte(1),
    USBD = usbd(),
}

fn gpiote() {
    with_state(|state| {
        for (i, button) in state.buttons.iter_mut().enumerate() {
            let id = Id::new(i).unwrap();
            if channel(&state.gpiote, id).is_event_triggered() {
                let pressed = button.pin.is_low().unwrap();
                state.events.push(board::button::Event { button: id, pressed }.into());
            }
        }
        state.gpiote.reset_events();
    });
}

fn radio() {
    with_state(|state| {
        if let Some(next_update) =
            state.ble_radio.recv_beacon_interrupt(state.ble_timer.now(), &mut state.ble_scanner)
        {
            state.ble_timer.configure_interrupt(next_update);
            critical_section::with(|cs| {
                if let Some(packet) = BLE_PACKET.take(cs) {
                    if state.ble_packet_queue.len() < 50 {
                        state.ble_packet_queue.push_back(packet);
                        state.events.push(board::radio::Event::Received.into());
                        log::warn!("BLE queue size: {}", state.ble_packet_queue.len());
                    } else {
                        log::warn!(
                            "BLE Packet dropped. Queue size: {}",
                            state.ble_packet_queue.len()
                        );
                    }
                }
            });
        }
    })
}

fn radio_timer() {
    with_state(|state| {
        if !state.ble_timer.is_interrupt_pending() {
            return;
        }
        state.ble_timer.clear_interrupt();

        let cmd = state.ble_scanner.timer_update(state.ble_timer.now());
        state.ble_radio.configure_receiver(cmd.radio);
        state.ble_timer.configure_interrupt(cmd.next_update);
    });
}

fn timer(timer: usize) {
    let timer = Id::new(timer).unwrap();
    with_state(|state| {
        state.events.push(board::timer::Event { timer }.into());
        state.timers.tick(*timer);
    })
}

fn uarte(uarte: usize) {
    let uart = Id::new(uarte).unwrap();
    with_state(|state| {
        state.uarts.tick(uart, |event| state.events.push(event.into()));
    })
}

fn usbd() {
    with_state(|state| {
        let polled = state.usb_dev.poll(&mut [state.serial.port()]);
        state.serial.tick(polled, |event| state.events.push(event.into()));
    });
}
