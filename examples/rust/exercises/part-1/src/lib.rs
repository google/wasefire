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

//! This exercise is essentially some "echo" applet on a UART. It writes back to the UART whatever
//! it receives.
//!
//! If you are running the applet on the host runner, you can connect to the UART with:
//!
//! ```shell
//! socat -,cfmakeraw UNIX-CONNECT:target/wasefire/uart0
//! ```

#![no_std]
wasefire::applet!();

fn main() {
    loop {
        // TODO: Read from the UART using serial::read_any().
        // TODO: Debug print what was read with debug!().
        // TODO: Send back to the UART what was read with serial::write_all().
        // TODO: Debug print that the data was sent.
    }
}
