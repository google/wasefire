// Copyright 2024 Google LLC
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
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Debug, Display};
use std::io::Write;
use std::time::Duration;

use anyhow::{anyhow, ensure, Context};
use rusb::{Device, DeviceHandle, Error, TransferType, UsbContext};
use wasefire_logger as log;
use wasefire_protocol::DynFuture;

use crate::common::{Decoder, Encoder};

/// Returns the only supported device.
///
/// If there are multiple supported device, asks the user to select one.
///
/// The context may be `rusb::GlobalContext::default()`.
pub fn choose_device<T: UsbContext>(context: &T) -> anyhow::Result<Candidate<T>> {
    let mut candidates = list(context)?;
    let candidate = candidates.pop().ok_or_else(|| anyhow!("no device found"))?;
    if candidates.is_empty() {
        log::info!("Using the only candidate found: {candidate}");
        return Ok(candidate);
    }
    candidates.push(candidate);
    std::println!("Found multiple devices:");
    for (index, candidate) in candidates.iter().enumerate() {
        // TODO: We should connect to each using the platform protocol to ask for the serial.
        std::println!("[{index}] {candidate}");
    }
    std::print!("Type the device index: ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    assert_eq!(line.pop(), Some('\n'));
    let index = line.parse()?;
    ensure!(index < candidates.len(), "index out of bounds");
    Ok(candidates.swap_remove(index))
}

/// Returns the list of supported devices.
///
/// The context may be `rusb::GlobalContext::default()`.
pub fn list<T: UsbContext>(context: &T) -> anyhow::Result<Vec<Candidate<T>>> {
    let mut candidates = Vec::new();
    for device in context.devices()?.iter() {
        match is_wasefire(device.clone()) {
            Ok(Some(candidate)) => candidates.push(candidate),
            Ok(None) => (),
            Err(error) => log::error!("{:?}: {:?}", device, error),
        }
    }
    Ok(candidates)
}

fn is_wasefire<T: UsbContext>(device: Device<T>) -> anyhow::Result<Option<Candidate<T>>> {
    let device_descriptor = device.device_descriptor().context("device desc")?;
    // Wasefire defines the class at interface-level, not device-level.
    if device_descriptor.class_code() != 0 {
        return Ok(None);
    }
    let num_configurations = device_descriptor.num_configurations();
    for config_index in 0 .. num_configurations {
        let config_descriptor = device.config_descriptor(config_index).context("config desc")?;
        for interface in config_descriptor.interfaces() {
            for descriptor in interface.descriptors() {
                // Wasefire uses the FF vendor class and 58 sub class (chosen at random).
                if descriptor.class_code() != 0xff || descriptor.sub_class_code() != 0x58 {
                    continue;
                }
                // Wasefire doesn't use alternate setting number.
                if descriptor.setting_number() != 0 {
                    continue;
                }
                // Wasefire expects 2 bulk endpoints with max-packet-size of 64.
                let mut flags = 0; // IN=1 OUT=2
                for endpoint in descriptor.endpoint_descriptors() {
                    if endpoint.transfer_type() != TransferType::Bulk {
                        continue;
                    }
                    if endpoint.max_packet_size() != MAX_PACKET_SIZE as u16 {
                        continue;
                    }
                    flags |= match endpoint.address() {
                        EP_IN => 1,
                        EP_OUT => 2,
                        _ => continue,
                    };
                }
                if flags != 3 || descriptor.num_endpoints() != 2 {
                    continue;
                }
                let configuration = config_descriptor.number();
                let interface = descriptor.interface_number();
                return Ok(Some(Candidate { device, configuration, interface }));
            }
        }
    }
    Ok(None)
}

/// Devices that look like Wasefire.
///
/// Those are good candidates to connect to.
#[derive(Clone)]
pub struct Candidate<T: UsbContext> {
    device: Device<T>,
    configuration: u8,
    interface: u8,
}

impl<T: UsbContext> Display for Candidate<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Candidate { device, configuration, interface } = self;
        write!(f, "{device:?} (conf:{configuration} iface:{interface})")
    }
}

impl<T: UsbContext> Candidate<T> {
    /// Creates a connection to the candidate device.
    ///
    /// The timeout is used for all send and receive operations using this connection.
    pub fn connect(self, timeout: Duration) -> rusb::Result<Connection<T>> {
        let Candidate { device, configuration, interface } = self;
        let handle = device.open()?;
        let current_configuration = handle.active_configuration()?;
        if current_configuration != configuration {
            log::info!("Configuring the device from {current_configuration} to {configuration}.");
            handle.set_active_configuration(configuration)?;
        }
        handle.claim_interface(interface)?;
        Ok(Connection { device, handle, timeout })
    }
}

/// Holds a connection to a device.
pub struct Connection<T: UsbContext> {
    device: Device<T>,
    handle: DeviceHandle<T>,
    timeout: Duration,
}

impl<T: UsbContext> Display for Connection<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.device.fmt(f)
    }
}

impl<T: UsbContext> Connection<T> {
    /// Provides access to the USB device.
    pub fn device(&self) -> &Device<T> {
        &self.device
    }

    /// Sets the timeout for subsequent send and receive operations.
    ///
    /// The default timeout is 1 second.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// FOR TESTING ONLY: Writes a packet directly to the endpoint.
    pub fn testonly_write(&self, packet: &[u8]) -> rusb::Result<usize> {
        self.handle.write_bulk(EP_OUT, packet, self.timeout)
    }

    /// FOR TESTING ONLY: Reads a packet directly from the endpoint.
    pub fn testonly_read(&self, packet: &mut [u8]) -> rusb::Result<usize> {
        self.handle.read_bulk(EP_IN, packet, self.timeout)
    }
}

impl<T: UsbContext> wasefire_protocol::Connection for Connection<T> {
    fn write<'a>(&'a mut self, request: &'a [u8]) -> DynFuture<'a, ()> {
        Box::pin(async move {
            for packet in Encoder::new(request) {
                let written = self.handle.write_bulk(EP_OUT, &packet, self.timeout)?;
                if written != MAX_PACKET_SIZE {
                    log::error!("Sent only {written} bytes instead of {MAX_PACKET_SIZE}.");
                    return Err(Error::Io.into());
                }
            }
            Ok(())
        })
    }

    fn read(&mut self) -> DynFuture<Box<[u8]>> {
        Box::pin(async move {
            let mut decoder = Decoder::default();
            loop {
                let mut packet = [0; MAX_PACKET_SIZE];
                let read = self.handle.read_bulk(EP_IN, &mut packet, self.timeout)?;
                if read != MAX_PACKET_SIZE {
                    log::error!("Received only {read} bytes instead of {MAX_PACKET_SIZE}.");
                    return Err(Error::Io.into());
                }
                match decoder.push(&packet) {
                    Some(Ok(x)) => return Ok(x.into_boxed_slice()),
                    Some(Err(x)) => decoder = x,
                    None => {
                        log::error!("Could not decode packet {packet:02x?}.");
                        return Err(Error::Io.into());
                    }
                }
            }
        })
    }
}

const MAX_PACKET_SIZE: usize = 64;
const EP_OUT: u8 = 0x01;
const EP_IN: u8 = 0x81;
