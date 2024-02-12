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

//! Provides a bridge between UART and USB serial.

#![no_std]
wasefire::applet!();

fn main() {
    let uart = Serial { name: "UART", port: uart::Uart::new(0).unwrap() };
    let usb = Serial { name: "USB", port: usb::serial::UsbSerial };
    let _uart = serial::Listener::new(&uart.port, serial::Event::Read);
    let _usb = serial::Listener::new(&usb.port, serial::Event::Read);
    let mut buffer = [0; 1024];
    loop {
        uart.copy_to(&usb, &mut buffer);
        usb.copy_to(&uart, &mut buffer);
        scheduling::wait_for_callback();
    }
}

struct Serial<Port> {
    name: &'static str,
    port: Port,
}

impl<Src: serial::Serial> Serial<Src> {
    fn copy_to<Dst: serial::Serial>(&self, dst: &Serial<Dst>, buf: &mut [u8]) {
        let mut total = 0;
        while total % buf.len() == 0 {
            let len = serial::read(&self.port, buf).unwrap();
            if len == 0 {
                break;
            }
            let wlen = serial::write(&dst.port, &buf[.. len]).unwrap();
            if wlen < len {
                debug!("Only wrote {wlen} from {len} bytes to {}.", dst.name);
            }
            total += len;
        }
        if 0 < total {
            serial::flush(&dst.port).unwrap();
            debug!("Copied {total} bytes from {} to {}.", self.name, dst.name);
        }
    }
}
