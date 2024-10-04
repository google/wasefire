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

use std::process::{Child, Command};
use std::time::Duration;

use anyhow::{ensure, Result};
use usb_device::class::UsbClass;
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::StringDescriptors;
use usb_device::prelude::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usb_device::UsbError;
use usbd_serial::SerialPort;
use usbip_device::UsbIpBus;
use wasefire_board_api::usb::serial::Serial;
use wasefire_board_api::usb::Api;
use wasefire_error::{Code, Error};
use wasefire_protocol_usb::Rpc;

use crate::with_state;

mod serial;

pub enum Impl {}

impl Api for Impl {
    type Serial = serial::Impl;
}

pub struct State {
    protocol: Option<Rpc<'static, UsbIpBus>>,
    serial: Option<Serial<'static, UsbIpBus>>,
    // Some iff at least one class is Some.
    usb_dev: Option<UsbDevice<'static, UsbIpBus>>,
}

pub fn init() -> Result<()> {
    if with_state(|x| x.usb.usb_dev.is_none()) {
        return Ok(());
    }
    ensure!(
        spawn(&["sudo", "modprobe", "vhci-hcd"]).wait().unwrap().code() == Some(0),
        "failed to load kernel module for USB/IP"
    );
    let mut usbip = spawn(&["sudo", "usbip", "attach", "-r", "localhost", "-b", "1-1"]);
    loop {
        with_state(|state| state.usb.poll());
        match usbip.try_wait().unwrap() {
            None => continue,
            Some(e) => ensure!(e.code() == Some(0), "failed to attach remote USB/IP device"),
        }
        break;
    }
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1)).await;
            with_state(|state| {
                let polled = state.usb.poll();
                let crate::board::State { sender, usb: State { protocol, serial, .. }, .. } = state;
                if let Some(protocol) = protocol {
                    protocol.tick(|event| drop(sender.try_send(event.into())));
                }
                if let Some(serial) = serial {
                    let has_serial =
                        !matches!(serial.port().read(&mut []), Err(UsbError::WouldBlock));
                    serial.tick(polled && has_serial, |event| drop(sender.try_send(event.into())));
                }
            });
        }
    });
    Ok(())
}

impl State {
    pub fn new(vid_pid: &str, protocol: bool, serial: bool) -> Self {
        let mut state = State { protocol: None, serial: None, usb_dev: None };
        if !protocol && !serial {
            return state;
        }
        let usb_bus = Box::leak(Box::new(UsbBusAllocator::new(UsbIpBus::new())));
        if protocol {
            state.protocol = Some(Rpc::new(usb_bus));
        }
        if serial {
            state.serial = Some(Serial::new(SerialPort::new(usb_bus)));
        }
        let (vid, pid) = vid_pid.split_once(':').expect("--usb-vid-pid must be VID:PID");
        let vid = u16::from_str_radix(vid, 16).expect("invalid VID");
        let pid = u16::from_str_radix(pid, 16).expect("invalid PID");
        state.usb_dev = Some(
            UsbDeviceBuilder::new(usb_bus, UsbVidPid(vid, pid))
                .strings(&[StringDescriptors::new(usb_device::LangID::EN).product("Wasefire")])
                .unwrap()
                .build(),
        );
        state
    }

    pub fn protocol(&mut self) -> &mut Rpc<'static, UsbIpBus> {
        self.protocol.as_mut().unwrap()
    }

    pub fn serial(&mut self) -> Result<&mut Serial<'static, UsbIpBus>, Error> {
        self.serial.as_mut().ok_or(Error::world(Code::NotImplemented))
    }

    fn poll(&mut self) -> bool {
        let usb_dev = self.usb_dev.as_mut().unwrap();
        let mut classes = Vec::<&mut dyn UsbClass<_>>::with_capacity(2);
        classes.extend(self.protocol.as_mut().map(|x| x as &mut dyn UsbClass<_>));
        classes.extend(self.serial.as_mut().map(|x| x.port() as &mut dyn UsbClass<_>));
        usb_dev.poll(&mut classes)
    }
}

fn spawn(cmd: &[&str]) -> Child {
    println!("Executing: {}", cmd.join(" "));
    Command::new(cmd[0]).args(&cmd[1 ..]).spawn().unwrap()
}
