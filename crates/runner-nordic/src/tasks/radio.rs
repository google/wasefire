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

use rubble::bytes::ByteWriter;
use rubble::link::{NextUpdate, RadioCmd};
use rubble::time::{Duration, Timer};
use wasefire_board_api::radio::Api;
use wasefire_board_api::{Error, Support};
use wasefire_logger as logger;

use crate::with_state;

pub enum Impl {}

impl Api for Impl {
    fn enable() -> Result<(), Error> {
        with_state(|state| {
            let scanner_cmd =
                state.ble_scanner.configure(state.ble_timer.now(), Duration::millis(500));
            state.ble_radio.configure_receiver(scanner_cmd.radio);
            state.ble_timer.configure_interrupt(scanner_cmd.next_update);
            Ok(())
        })
    }

    fn disable() -> Result<(), Error> {
        with_state(|state| {
            state.ble_timer.configure_interrupt(NextUpdate::Disable);
            state.ble_radio.configure_receiver(RadioCmd::Off);
            Ok(())
        })
    }

    fn read(output: &mut [u8]) -> Result<usize, Error> {
        with_state(|state| {
            let packet = state.ble_packet_queue.pop_front().ok_or(Error::World)?;
            let len = packet.len();
            let mut writer = ByteWriter::new(output);
            // Serialize RadioMetadata first, then BDADDR and append the packet.
            let res: Result<(), rubble::Error> = try {
                writer.write_u32_le(packet.metadata.ticks)?;
                writer.write_u16_le(packet.metadata.freq)?;
                writer.write_slice(&packet.metadata.rssi.to_le_bytes())?;
                writer.write_u8(packet.metadata.pdu_type)?;
                writer.write_slice(&packet.addr)?;
                writer.write_slice(&packet.data)?;
            };
            res.map_err(|_| Error::World)?;
            logger::trace!("{}{:?} = read({})", len, &output[.. len], output.len());
            Ok(len)
        })
    }
}

impl Supported for Impl {}
