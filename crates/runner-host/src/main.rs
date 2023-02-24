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

#![feature(try_blocks)]

use std::io::BufRead;
use std::path::Path;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use tokio::runtime::Handle;
use tokio::sync::mpsc::channel;
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};
use usb_device::UsbError;
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use usbip_device::UsbIpBus;
use wasefire_board_api::usb::serial::Serial;
use wasefire_scheduler::Scheduler;
use wasefire_store::{FileOptions, FileStorage};

use crate::board::timer::Timers;
use crate::board::State;

mod board;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    // TODO: Should be a flag controlled by xtask (value is duplicated there).
    const STORAGE: &str = "../../target/storage.bin";
    let usb_bus = Box::leak(Box::new(UsbBusAllocator::new(UsbIpBus::new())));
    let serial = Serial::new(SerialPort::new(usb_bus));
    let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .product("Serial port")
        .device_class(USB_CLASS_CDC)
        .build();
    let options = FileOptions { word_size: 4, page_size: 4096, num_pages: 16 };
    let storage = Some(FileStorage::new(Path::new(STORAGE), options).unwrap());
    let (sender, receiver) = channel(10);
    let state = Arc::new(Mutex::new(board::State {
        sender,
        button: false,
        led: false,
        timers: Timers::default(),
        serial,
        usb_dev,
        storage,
    }));
    assert_eq!(spawn(&["sudo", "modprobe", "vhci-hcd"]).wait().unwrap().code().unwrap(), 0);
    let mut usbip = spawn(&["sudo", "usbip", "attach", "-r", "localhost", "-b", "1-1"]);
    loop {
        state.lock().unwrap().poll();
        match usbip.try_wait().unwrap() {
            None => continue,
            Some(e) => assert_eq!(e.code().unwrap(), 0),
        }
        break;
    }
    tokio::spawn({
        let state = state.clone();
        async move {
            loop {
                tokio::time::sleep(Duration::from_millis(1)).await;
                let mut state = state.lock().unwrap();
                let polled = state.poll()
                    && !matches!(state.serial.port().read(&mut []), Err(UsbError::WouldBlock));
                let State { sender, serial, .. } = &mut *state;
                serial.tick(polled, |event| drop(sender.try_send(event.into())));
            }
        }
    });
    tokio::spawn({
        let state = state.clone();
        async move {
            for line in std::io::stdin().lock().lines() {
                let pressed = match line.unwrap().as_str() {
                    "button" => None,
                    "press" => Some(true),
                    "release" => Some(false),
                    x => {
                        println!("Unrecognized command: {x}");
                        continue;
                    }
                };
                let mut state = state.lock().unwrap();
                board::button::event(&mut state, pressed);
            }
        }
    });
    println!("Running.");
    Handle::current().spawn_blocking(|| Scheduler::run(board::Board { receiver, state })).await?
}

fn spawn(cmd: &[&str]) -> Child {
    Command::new(cmd[0]).args(&cmd[1 ..]).spawn().unwrap()
}
