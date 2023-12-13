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

use rubble::beacon::ScanCallback;
use rubble::bytes::ByteReader;
use rubble::link::{Cmd, Metadata, NextUpdate, RadioCmd};
use rubble::link::advertising::{Header, Pdu};
use rubble::link::filter::{self, AddressFilter, ScanFilter};
use rubble::phy::AdvertisingChannel;
use rubble::time::{Duration, Instant};

/// A passive scanner for BLE trackers advertisements.
/// Based on the Beacon scanner in rubble but incorporate some tricks
/// to handle BLE trackers.
pub struct BleTrackerScanner<C: ScanCallback, F: AddressFilter> {
    cb: C,
    filter: ScanFilter<F>,
    interval: Duration,
    channel: AdvertisingChannel,
}

impl<C: ScanCallback> BleTrackerScanner<C, filter::AllowAll> {
    /// Creates a `BleTrackerScanner` that will report beacons from any device.
    pub fn new(callback: C) -> Self {
        Self::with_filter(callback, filter::AllowAll)
    }
}

impl<C: ScanCallback, F: AddressFilter> BleTrackerScanner<C, F> {
    /// Creates a `BleTrackerScanner` with a custom device filter.
    pub fn with_filter(callback: C, scan_filter: F) -> Self {
        Self {
            cb: callback,
            filter: ScanFilter::new(scan_filter),
            interval: Duration::micros(0),
            channel: AdvertisingChannel::first(),
        }
    }

    /// Configures the `BleTrackerScanner` and returns a `Cmd` to apply to the radio.
    ///
    /// The `next_update` field of the returned `Cmd` specifies when to call `timer_update` the next
    /// time. The timer used for this needs to be pretty accurate because of advanced advertising packets,
    /// which require to precisely switch channel to a data channel to capture the whole advertisement.
    /// Otherwise it is only used to switch to the next advertising channel after `interval` elapses.
    pub fn configure(&mut self, now: Instant, interval: Duration) -> Cmd {
        self.interval = interval;
        self.channel = AdvertisingChannel::first();

        Cmd {
            // Switch channels
            next_update: NextUpdate::At(now + self.interval),

            radio: RadioCmd::ListenAdvertising {
                channel: self.channel,
            },

            queued_work: false,
        }
    }

    /// Updates the `BleTrackerScanner` after the configured timer has fired.
    ///
    /// This switches to the next advertising channel and will listen there.
    pub fn timer_update(&mut self, now: Instant) -> Cmd {
        self.channel = self.channel.cycle();

        Cmd {
            // Switch channels
            next_update: NextUpdate::At(now + self.interval),

            radio: RadioCmd::ListenAdvertising {
                channel: self.channel,
            },

            queued_work: false,
        }
    }

    /// Processes a received advertising channel packet.
    ///
    /// This should be called whenever the radio receives a packet on the configured advertising
    /// channel.
    pub fn process_adv_packet(
        &mut self,
        header: Header,
        payload: &[u8],
        metadata: Metadata,
    ) -> Cmd {
        if metadata.crc_ok && header.type_().is_beacon() {
            // Partially decode to get the device ID and run it through the filter
            if let Ok(pdu) = Pdu::from_header_and_payload(header, &mut ByteReader::new(payload)) {
                if self.filter.should_scan(*pdu.sender()) {
                    let ad = pdu.advertising_data().unwrap();
                    self.cb.beacon(*pdu.sender(), ad, metadata);
                }
            }
        }

        Cmd {
            next_update: NextUpdate::Keep,
            radio: RadioCmd::ListenAdvertising {
                channel: self.channel,
            },
            queued_work: false,
        }
    }
}

