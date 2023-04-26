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
use std::sync::{Arc, Mutex};
use std::time::Duration;

use board::usb::serial::{HasSerial, WithSerial};
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::prelude::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usb_device::UsbError;
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use usbip_device::UsbIpBus;
use wasefire_board_api as board;
use wasefire_board_api::usb::serial::Serial;

use crate::board::{Board, State};

impl board::usb::Api for &mut Board {
    type Serial<'a> = WithSerial<&'a mut Board>
    where Self: 'a;
    fn serial(&mut self) -> Self::Serial<'_> {
        WithSerial(self)
    }
}

impl HasSerial for &mut Board {
    type UsbBus = UsbIpBus;

    fn with_serial<R>(&mut self, f: impl FnOnce(&mut Serial<Self::UsbBus>) -> R) -> R {
        f(&mut self.state.lock().unwrap().usb.serial)
    }
}

pub struct Usb {
    pub serial: Serial<'static, UsbIpBus>,
    pub usb_dev: UsbDevice<'static, UsbIpBus>,
}

impl Default for Usb {
    fn default() -> Self {
        let usb_bus = Box::leak(Box::new(UsbBusAllocator::new(UsbIpBus::new())));
        let serial = Serial::new(SerialPort::new(usb_bus));
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
            .product("Serial port")
            .device_class(USB_CLASS_CDC)
            .build();
        Self { serial, usb_dev }
    }
}

impl Usb {
    pub fn init(state: Arc<Mutex<State>>) {
        assert_eq!(spawn(&["sudo", "modprobe", "vhci-hcd"]).wait().unwrap().code().unwrap(), 0);
        let mut usbip = spawn(&["sudo", "usbip", "attach", "-r", "localhost", "-b", "1-1"]);
        loop {
            state.lock().unwrap().usb.poll();
            match usbip.try_wait().unwrap() {
                None => continue,
                Some(e) => assert_eq!(e.code().unwrap(), 0),
            }
            break;
        }
        tokio::spawn({
            async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    let mut state = state.lock().unwrap();
                    let polled = state.usb.poll()
                        && !matches!(
                            state.usb.serial.port().read(&mut []),
                            Err(UsbError::WouldBlock)
                        );
                    let State { sender, usb: Usb { serial, .. }, .. } = &mut *state;
                    serial.tick(polled, |event| drop(sender.try_send(event.into())));
                }
            }
        });
    }

    pub fn poll(&mut self) -> bool {
        self.usb_dev.poll(&mut [self.serial.port()])
    }
}

fn spawn(cmd: &[&str]) -> Child {
    Command::new(cmd[0]).args(&cmd[1 ..]).spawn().unwrap()
}
