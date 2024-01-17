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

use alloc::string::String;

use interface::{deserialize, serialize, Request, Response};
use wasefire::{debug, scheduling, serial};

pub struct Serial<'a, T: serial::Serial> {
    serial: &'a T,
    reader: serial::DelimitedReader<'a, T>,
}

impl<'a, T: serial::Serial> Serial<'a, T> {
    pub fn new(serial: &'a T) -> Self {
        // Requests are delimited by bytes equal to zero.
        let reader = serial::DelimitedReader::new(serial, 0);
        Serial { serial, reader }
    }

    pub fn receive(&mut self) -> Request {
        // Wait until we receive a full request.
        let mut request = loop {
            if let Some(x) = self.reader.next_frame() {
                break x;
            }
            scheduling::wait_for_callback();
        };
        // Ignore all requests but the last one.
        while let Some(x) = self.reader.next_frame() {
            debug!("Dropping old request.");
            request = x;
        }
        deserialize(&mut request)
    }

    pub fn send(&self, response: Result<Response, String>) {
        match serial::write_all(self.serial, &serialize(&response)) {
            Ok(()) => (),
            Err(err) => debug!("Sending response failed with {err}"),
        }
    }
}
