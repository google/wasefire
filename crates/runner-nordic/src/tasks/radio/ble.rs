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

use rubble::link::{NextUpdate, RadioCmd};
use rubble::time::{Duration, Timer};
use wasefire_applet_api::radio::ble::Advertisement;
use wasefire_board_api::radio::ble::{Api, Event};
use wasefire_board_api::{Error, Supported};
use wasefire_logger as log;

use crate::with_state;

pub enum Impl {}

impl Api for Impl {
    fn enable(event: &Event) -> Result<(), Error> {
        match event {
            Event::Advertisement => with_state(|state| {
                let scanner_cmd =
                    state.ble_scanner.configure(state.ble_timer.now(), Duration::millis(500));
                state.ble_radio.configure_receiver(scanner_cmd.radio);
                state.ble_timer.configure_interrupt(scanner_cmd.next_update);
                Ok(())
            }),
        }
    }

    fn disable(event: &Event) -> Result<(), Error> {
        match event {
            Event::Advertisement => with_state(|state| {
                state.ble_timer.configure_interrupt(NextUpdate::Disable);
                state.ble_radio.configure_receiver(RadioCmd::Off);
                Ok(())
            }),
        }
    }

    fn read_advertisement(output: &mut Advertisement) -> Result<bool, Error> {
        with_state(|state| {
            let input = match state.ble_packet_queue.pop_front() {
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
            log::trace!("read_advertisement() -> {:?}", log::Debug2Format(output));
            Ok(true)
        })
    }
}

impl Supported for Impl {}
