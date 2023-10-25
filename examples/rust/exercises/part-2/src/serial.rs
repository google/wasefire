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
use wasefire::serial;

pub struct Serial<T: serial::Serial>(T);

impl<T: serial::Serial> Serial<T> {
    pub fn new(serial: T) -> Self {
        todo!()
    }

    pub fn receive(&self) -> Request {
        // TODO: Use deserialize()
        todo!()
    }

    pub fn send(&self, response: Result<Response, String>) {
        // TODO: Use serialize()
        todo!()
    }
}
