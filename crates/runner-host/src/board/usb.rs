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

use std::process::Stdio;
use std::time::Duration;

use anyhow::{Result, ensure};
use tokio::process::{Child, Command};
use usb_device::UsbError;
use usb_device::class::UsbClass;
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::StringDescriptors;
use usb_device::prelude::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usbd_hid::descriptor::{CtapReport, SerializedDescriptor};
use usbd_hid::hid_class::HIDClass;
use usbd_serial::SerialPort;
use usbip_device::UsbIpBus;
use wasefire_board_api::usb::Api;
use wasefire_board_api::usb::ctap::{Ctap, HasHid, WithHid};
use wasefire_board_api::usb::serial::{HasSerial, Serial, WithSerial};
use wasefire_cli_tools::cmd;
use wasefire_error::{Code, Error};
use wasefire_protocol_usb::Rpc;

use crate::with_state;

pub enum Impl {}

impl Api for Impl {
    type Ctap = WithHid<Impl>;
    type Serial = WithSerial<Impl>;
}

pub struct State {
    protocol: Option<Rpc<'static, UsbIpBus>>,
    ctap: Option<Ctap<'static, UsbIpBus>>,
    serial: Option<Serial<'static, UsbIpBus>>,
    // Some iff at least one class is Some.
    usb_dev: Option<UsbDevice<'static, UsbIpBus>>,
}

pub async fn init() -> Result<()> {
    if with_state(|x| x.usb.usb_dev.is_none()) {
        return Ok(());
    }
    if !has_mod("vhci_hcd").await? {
        ensure!(
            spawn(&["sudo", "modprobe", "vhci-hcd"])?.wait().await?.code() == Some(0),
            "failed to load kernel module for USB/IP"
        );
    }
    let mut usbip = spawn(&["sudo", "usbip", "attach", "-r", "localhost", "-b", "1-1"])?;
    loop {
        let fast = with_state(|state| state.usb.poll());
        let Some(status) = usbip.try_wait()? else {
            tokio::time::sleep(Duration::from_millis(if fast { 1 } else { 500 })).await;
            continue;
        };
        ensure!(status.code() == Some(0), "failed to attach remote USB/IP device");
        break;
    }
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1)).await;
            with_state(|state| {
                let polled = state.usb.poll();
                let crate::board::State {
                    sender, usb: State { protocol, ctap, serial, .. }, ..
                } = state;
                if let Some(protocol) = protocol {
                    protocol.tick(|event| drop(sender.try_send(event.into())));
                }
                if let Some(ctap) = ctap {
                    ctap.tick(|event| drop(sender.try_send(event.into())));
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
    pub fn new(vid_pid: &str, protocol: bool, ctap: bool, serial: bool) -> Self {
        let mut state = State { protocol: None, ctap: None, serial: None, usb_dev: None };
        if !protocol && !ctap && !serial {
            return state;
        }
        let usb_bus = Box::leak(Box::new(UsbBusAllocator::new(UsbIpBus::new())));
        if protocol {
            state.protocol = Some(Rpc::new(usb_bus));
        }
        if ctap {
            state.ctap = Some(Ctap::new(HIDClass::new(usb_bus, CtapReport::desc(), 255)));
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

    pub fn ctap(&mut self) -> Result<&mut Ctap<'static, UsbIpBus>, Error> {
        self.ctap.as_mut().ok_or(Error::world(Code::NotImplemented))
    }

    pub fn serial(&mut self) -> Result<&mut Serial<'static, UsbIpBus>, Error> {
        self.serial.as_mut().ok_or(Error::world(Code::NotImplemented))
    }

    fn poll(&mut self) -> bool {
        let usb_dev = self.usb_dev.as_mut().unwrap();
        let mut classes = Vec::<&mut dyn UsbClass<_>>::new();
        classes.extend(self.protocol.as_mut().map(|x| x as &mut dyn UsbClass<_>));
        classes.extend(self.ctap.as_mut().map(|x| x.class() as &mut dyn UsbClass<_>));
        classes.extend(self.serial.as_mut().map(|x| x.port() as &mut dyn UsbClass<_>));
        usb_dev.poll(&mut classes)
    }
}

impl HasHid for Impl {
    type UsbBus = UsbIpBus;

    fn with_hid<R>(f: impl FnOnce(&mut Ctap<Self::UsbBus>) -> R) -> R {
        with_state(|state| f(state.usb.ctap().unwrap()))
    }
}

impl HasSerial for Impl {
    type UsbBus = UsbIpBus;

    fn with_serial<R>(f: impl FnOnce(&mut Serial<Self::UsbBus>) -> R) -> R {
        with_state(|state| f(state.usb.serial().unwrap()))
    }
}

fn spawn(cmd: &[&str]) -> Result<Child> {
    println!("Executing: {}", cmd.join(" "));
    cmd::spawn(Command::new(cmd[0]).args(&cmd[1 ..]).stdin(Stdio::null()))
}

async fn has_mod(name: &str) -> Result<bool> {
    for line in cmd::output(&mut Command::new("lsmod")).await?.stdout.split(|&x| x == b'\n') {
        if let Some(suffix) = line.strip_prefix(name.as_bytes()) {
            if suffix.first() == Some(&b' ') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
