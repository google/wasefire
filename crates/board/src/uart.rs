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

//! UART interface.

use derivative::Derivative;

use crate::{Error, Id, Support, Unsupported};

/// UART event.
#[derive(Derivative)]
#[derivative(Debug(bound = ""), PartialEq(bound = ""), Eq(bound = ""))]
pub struct Event<B: crate::Api + ?Sized> {
    /// The UART that triggered the event.
    pub uart: Id<crate::Uart<B>>,

    /// Whether the UART may be ready for read or write.
    pub direction: Direction,
}

/// Whether it might be possible to read or write.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    /// There might be data to read.
    Read,

    /// It might be possible to write data.
    Write,
}

impl<B: crate::Api> From<Event<B>> for crate::Event<B> {
    fn from(event: Event<B>) -> Self {
        crate::Event::Uart(event)
    }
}

/// UART interface.
pub trait Api: Support<usize> + Send {
    /// Reads from a UART into a buffer.
    ///
    /// Returns the number of bytes read. It could be zero if there's nothing to read.
    fn read(uart: Id<Self>, output: &mut [u8]) -> Result<usize, Error>;

    /// Writes from a buffer to a UART.
    ///
    /// Returns the number of bytes written. It could be zero if the other side is not ready.
    fn write(uart: Id<Self>, input: &[u8]) -> Result<usize, Error>;

    /// Enables a given event to be triggered.
    fn enable(uart: Id<Self>, direction: Direction) -> Result<(), Error>;

    /// Disables a given event from being triggered.
    fn disable(uart: Id<Self>, direction: Direction) -> Result<(), Error>;
}

impl Api for Unsupported {
    fn read(_: Id<Self>, _: &mut [u8]) -> Result<usize, Error> {
        unreachable!()
    }

    fn write(_: Id<Self>, _: &[u8]) -> Result<usize, Error> {
        unreachable!()
    }

    fn enable(_: Id<Self>, _: Direction) -> Result<(), Error> {
        unreachable!()
    }

    fn disable(_: Id<Self>, _: Direction) -> Result<(), Error> {
        unreachable!()
    }
}
