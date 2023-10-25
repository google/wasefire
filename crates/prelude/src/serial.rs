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

//! Provides a generic API for serial communication.

use alloc::boxed::Box;
use core::cell::Cell;
use core::fmt::Debug;

use crate::scheduling;

/// Serial events to be notified.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Event {
    /// The serial may be ready to read.
    Read,

    /// The serial may be ready to write.
    Write,
}

/// Provides high-level serial API from low-level API.
pub trait Serial {
    type Error: Clone + Debug;

    /// Reads from the serial into a buffer without blocking.
    ///
    /// Returns how many bytes were read (and thus written to the buffer). This function does not
    /// block, so if there are no data available for read, zero is returned.
    fn read(&self, buffer: &mut [u8]) -> Result<usize, Self::Error>;

    /// Writes from a buffer to the serial.
    ///
    /// Returns how many bytes were written (and thus read from the buffer). This function does not
    /// block, so if the serial is not ready for write, zero is returned.
    fn write(&self, buffer: &[u8]) -> Result<usize, Self::Error>;

    /// Flushes the serial (in case reads or writes are buffered).
    fn flush(&self) -> Result<(), Self::Error>;

    /// Registers a callback for an event.
    ///
    /// # Safety
    ///
    /// The function pointer and data must live until unregistered. The function must support
    /// concurrent calls.
    unsafe fn register(
        &self, event: Event, func: extern "C" fn(*const u8), data: *const u8,
    ) -> Result<(), Self::Error>;

    /// Unregisters the callback for an event.
    fn unregister(&self, event: Event) -> Result<(), Self::Error>;
}

/// Reads from the serial into a buffer without blocking.
///
/// Returns how many bytes were read (and thus written to the buffer). This function does not
/// block, so if there are no data available for read, zero is returned.
pub fn read<T: Serial>(serial: &T, buffer: &mut [u8]) -> Result<usize, T::Error> {
    serial.read(buffer)
}

/// Synchronously reads at least one byte from a serial into a buffer.
///
/// This function will block if necessary.
pub fn read_any<T: Serial>(serial: &T, buffer: &mut [u8]) -> Result<usize, T::Error> {
    let mut reader = Reader::new(serial, buffer);
    scheduling::wait_until(|| !reader.is_empty());
    reader.result()
}

/// Synchronously reads from a serial into a buffer until it is filled.
///
/// This function will block if necessary.
pub fn read_all<T: Serial>(serial: &T, buffer: &mut [u8]) -> Result<(), T::Error> {
    let mut reader = Reader::new(serial, buffer);
    scheduling::wait_until(|| reader.is_done());
    reader.result()?;
    Ok(())
}

/// Synchronously reads exactly one byte.
pub fn read_byte<T: Serial>(serial: &T) -> Result<u8, T::Error> {
    let mut byte = 0;
    read_any(serial, core::slice::from_mut(&mut byte))?;
    Ok(byte)
}

/// Writes from a buffer to the serial.
///
/// Returns how many bytes were written (and thus read from the buffer). This function does not
/// block, so if the serial is not ready for write, zero is returned.
pub fn write<T: Serial>(serial: &T, buffer: &[u8]) -> Result<usize, T::Error> {
    serial.write(buffer)
}

/// Writes at least one byte from a buffer to a serial.
///
/// This function will block if necessary.
pub fn write_any<T: Serial>(serial: &T, buffer: &[u8]) -> Result<usize, T::Error> {
    let mut writer = Writer::new(serial, buffer);
    scheduling::wait_until(|| !writer.is_empty());
    writer.result()
}

/// Writes from a buffer to a serial until everything has been written.
///
/// This function will block if necessary.
pub fn write_all<T: Serial>(serial: &T, buffer: &[u8]) -> Result<(), T::Error> {
    let mut writer = Writer::new(serial, buffer);
    scheduling::wait_until(|| writer.is_done());
    writer.result()?;
    Ok(())
}

/// Flushes the serial (in case reads or writes are buffered).
pub fn flush<T: Serial>(serial: &T) -> Result<(), T::Error> {
    serial.flush()
}

/// Provides asynchronous read support.
#[must_use]
pub struct Reader<'a, T: Serial>(Listener<'a, T>);

impl<'a, T: Serial> Reader<'a, T> {
    /// Asynchronously reads from a serial into a buffer.
    pub fn new(serial: &'a T, buffer: &'a mut [u8]) -> Self {
        Reader(Listener::new(Kind::Reader { serial, buffer }))
    }

    /// Returns whether anything has been read (or an error occurred).
    pub fn is_empty(&mut self) -> bool {
        self.0.is_empty()
    }

    /// Returns whether everything has been read (or an error occurred).
    pub fn is_done(&mut self) -> bool {
        self.0.is_done()
    }

    /// Returns how many bytes were read (or if an error occurred).
    pub fn result(self) -> Result<usize, T::Error> {
        self.0.result()
    }
}

/// Provides asynchronous write support.
#[must_use]
pub struct Writer<'a, T: Serial>(Listener<'a, T>);

impl<'a, T: Serial> Writer<'a, T> {
    /// Asynchronously writes from a buffer to a serial.
    pub fn new(serial: &'a T, buffer: &'a [u8]) -> Self {
        Writer(Listener::new(Kind::Writer { serial, buffer }))
    }

    /// Returns whether anything has been written (or an error occurred).
    pub fn is_empty(&mut self) -> bool {
        self.0.is_empty()
    }

    /// Returns whether everything has been written (or an error occurred).
    pub fn is_done(&mut self) -> bool {
        self.0.is_done()
    }

    /// Returns how many bytes were written (or if an error occurred).
    pub fn result(self) -> Result<usize, T::Error> {
        self.0.result()
    }
}

struct Listener<'a, T: Serial> {
    kind: Kind<'a, T>,
    // Whether the callback triggered since last operation.
    ready: &'static Cell<bool>,
    // The callback is registered as long as not done.
    result: Result<usize, T::Error>,
}

impl<'a, T: Serial> Listener<'a, T> {
    fn new(kind: Kind<'a, T>) -> Self {
        let ready = Box::leak(Box::new(Cell::new(true)));
        let mut listener = Listener { kind, ready, result: Ok(0) };
        if listener.is_registered() {
            let event = listener.kind.event();
            let func = Self::call;
            let data = ready.as_ptr() as *const u8;
            let serial = listener.kind.serial();
            unsafe { serial.register(event, func, data) }.unwrap();
        }
        let _ = listener.update();
        listener
    }

    fn is_empty(&mut self) -> bool {
        matches!(self.update(), Ok(0))
    }

    fn is_done(&mut self) -> bool {
        let _ = self.update();
        !self.is_registered()
    }

    fn result(mut self) -> Result<usize, T::Error> {
        self.update()
    }

    fn update(&mut self) -> Result<usize, T::Error> {
        if !self.is_registered() || !self.ready.replace(false) {
            return self.result.clone();
        }
        let pos = self.result.as_mut().unwrap();
        match self.kind.update(*pos) {
            Ok(len) => *pos += len,
            err => self.result = err,
        }
        if !self.is_registered() {
            self.unregister();
        }
        self.result.clone()
    }

    fn is_registered(&self) -> bool {
        matches!(self.result, Ok(len) if len < self.kind.len())
    }

    fn unregister(&self) {
        self.kind.serial().unregister(self.kind.event()).unwrap()
    }

    extern "C" fn call(data: *const u8) {
        let ready = unsafe { &*(data as *const Cell<bool>) };
        ready.set(true);
    }
}

impl<'a, T: Serial> Drop for Listener<'a, T> {
    fn drop(&mut self) {
        if self.is_registered() {
            self.unregister();
        }
        drop(unsafe { Box::from_raw(self.ready.as_ptr()) });
    }
}

enum Kind<'a, T: Serial> {
    Reader { serial: &'a T, buffer: &'a mut [u8] },
    Writer { serial: &'a T, buffer: &'a [u8] },
}

impl<'a, T: Serial> Kind<'a, T> {
    fn event(&self) -> Event {
        match self {
            Kind::Reader { .. } => Event::Read,
            Kind::Writer { .. } => Event::Write,
        }
    }

    fn serial(&self) -> &T {
        match self {
            Kind::Reader { serial, .. } | Kind::Writer { serial, .. } => serial,
        }
    }

    fn len(&self) -> usize {
        match self {
            Kind::Reader { buffer, .. } => buffer.len(),
            Kind::Writer { buffer, .. } => buffer.len(),
        }
    }

    fn update(&mut self, pos: usize) -> Result<usize, T::Error> {
        match self {
            Kind::Reader { serial, buffer } => serial.read(&mut buffer[pos ..]),
            Kind::Writer { serial, buffer } => serial.write(&buffer[pos ..]),
        }
    }
}
