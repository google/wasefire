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

//! USB interface.

use crate::{Unimplemented, Unsupported};

pub mod serial;

/// USB event.
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// Serial event.
    Serial(serial::Event),
}

impl From<Event> for crate::Event {
    fn from(event: Event) -> Self {
        crate::Event::Usb(event)
    }
}

/// USB interface.
pub trait Api {
    type Serial<'a>: serial::Api
    where Self: 'a;
    fn serial(&mut self) -> Self::Serial<'_>;
}

impl Api for Unimplemented {
    type Serial<'a> = Unimplemented;
    fn serial(&mut self) -> Self::Serial<'_> {
        unreachable!()
    }
}

impl Api for Unsupported {
    type Serial<'a> = Unsupported;
    fn serial(&mut self) -> Self::Serial<'_> {
        Unsupported
    }
}
