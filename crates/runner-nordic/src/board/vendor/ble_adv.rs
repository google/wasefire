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

use alloc::collections::VecDeque;

use ble_adv::Advertisement;
use nrf52840_hal::pac::TIMER0;
use rubble::beacon::{BeaconScanner, ScanCallback};
use rubble::bytes::{ByteWriter, ToBytes};
use rubble::link::ad_structure::AdStructure;
use rubble::link::filter::AllowAll;
use rubble::link::{DeviceAddress, MIN_PDU_BUF, Metadata, NextUpdate, RadioCmd};
use rubble::time::{Duration, Timer};
use rubble_nrf5x::radio::BleRadio;
use rubble_nrf5x::timer::BleTimer;
use wasefire_board_api::Failure;
use wasefire_board_api::applet::{Handlers, Memory, MemoryExt as _};
use wasefire_board_api::vendor::Event;
use wasefire_error::Error;
use wasefire_logger as log;
use wasefire_sync::TakeCell;

use crate::with_state;

pub fn syscall(
    memory: impl Memory, mut handlers: impl Handlers<super::Key>, x2: u32, x3: u32, x4: u32,
) -> Result<u32, Failure> {
    match (x2, x3, x4) {
        (1, func, data) => {
            handlers.register(super::Key::BleAdv, func, data)?;
            enable()?;
            Ok(0)
        }
        (2, 0, 0) => {
            disable()?;
            handlers.unregister(super::Key::BleAdv)?;
            Ok(0)
        }
        (3, ptr, 0) => Ok(read_advertisement(memory.from_bytes_mut(ptr)?)? as u32),
        _ => Err(Failure::TRAP),
    }
}

fn enable() -> Result<(), Error> {
    with_state(|state| {
        let ble = &mut state.ble;
        let scanner_cmd = ble.scanner.configure(ble.timer.now(), Duration::millis(500));
        ble.radio.configure_receiver(scanner_cmd.radio);
        ble.timer.configure_interrupt(scanner_cmd.next_update);
        Ok(())
    })
}

pub fn disable() -> Result<(), Error> {
    with_state(|state| {
        let ble = &mut state.ble;
        ble.timer.configure_interrupt(NextUpdate::Disable);
        ble.radio.configure_receiver(RadioCmd::Off);
        Ok(())
    })
}

fn read_advertisement(output: &mut Advertisement) -> Result<bool, Error> {
    with_state(|state| {
        let ble = &mut state.ble;
        let input = match ble.packet_queue.pop_front() {
            Some(x) => x,
            None => return Ok(false),
        };
        output.ticks = input.metadata.ticks;
        output.freq = input.metadata.freq;
        output.rssi = input.metadata.rssi;
        output.pdu_type = input.metadata.pdu_type;
        output.addr = input.addr;
        output.data_len = input.data.len() as u8;
        output.data[.. input.data.len()].copy_from_slice(&input.data);
        log::trace!("read_advertisement() -> {:?}", output);
        Ok(true)
    })
}

pub struct Ble {
    radio: BleRadio,
    scanner: BeaconScanner<BleAdvScanCallback, AllowAll>,
    timer: BleTimer<TIMER0>,
    packet_queue: VecDeque<BlePacket>,
}

impl Ble {
    pub fn new(radio: BleRadio, timer: TIMER0) -> Self {
        let timer = BleTimer::init(timer);
        let scanner = BeaconScanner::new(BleAdvScanCallback);
        let packet_queue = VecDeque::<BlePacket>::new();
        Ble { radio, scanner, timer, packet_queue }
    }

    pub fn tick(&mut self, mut push: impl FnMut(Event<crate::Board>)) {
        let next_update =
            match self.radio.recv_beacon_interrupt(self.timer.now(), &mut self.scanner) {
                Some(x) => x,
                None => return,
            };
        self.timer.configure_interrupt(next_update);
        if let Some(packet) = BLE_PACKET.get() {
            if self.packet_queue.len() < 50 {
                self.packet_queue.push_back(packet);
                push(Event(super::Event::BleAdv));
                log::debug!("BLE queue size: {}", self.packet_queue.len());
            } else {
                log::warn!("BLE Packet dropped. Queue size: {}", self.packet_queue.len());
            }
        }
    }

    pub fn tick_timer(&mut self) {
        if !self.timer.is_interrupt_pending() {
            return;
        }
        self.timer.clear_interrupt();
        let cmd = self.scanner.timer_update(self.timer.now());
        self.radio.configure_receiver(cmd.radio);
        self.timer.configure_interrupt(cmd.next_update);
    }
}

static BLE_PACKET: TakeCell<BlePacket> = TakeCell::new(None);

struct RadioMetadata {
    ticks: u32,
    freq: u16,
    rssi: i8,
    pdu_type: u8,
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

struct BlePacket {
    addr: [u8; 6],
    metadata: RadioMetadata,
    data: alloc::vec::Vec<u8>,
}

struct BleAdvScanCallback;

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
        BLE_PACKET.put(packet);
    }
}
