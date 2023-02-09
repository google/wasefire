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

//! Provides API for USB serial.

use alloc::boxed::Box;
use core::cell::Cell;

use api::usb::serial as api;

use crate::scheduling;
use crate::usb::{convert, Error};

/// Reads from USB serial into a buffer without blocking.
///
/// Returns how many bytes were read (and thus written to the buffer). This function does not block,
/// so if there are no data available for read, zero is returned.
pub fn read(buf: &mut [u8]) -> Result<usize, Error> {
    let params = api::read::Params { ptr: buf.as_mut_ptr(), len: buf.len() };
    let api::read::Results { len } = unsafe { api::read(params) };
    convert(len)
}

/// Synchronously reads at least one byte from USB serial into a buffer.
///
/// This function will block if necessary.
pub fn read_any(buf: &mut [u8]) -> Result<usize, Error> {
    let mut reader = Reader::new(buf);
    scheduling::wait_until(|| !reader.is_empty());
    reader.result()
}

/// Synchronously reads from USB serial into a buffer until it is filled.
///
/// This function will block if necessary.
pub fn read_all(buf: &mut [u8]) -> Result<(), Error> {
    let mut reader = Reader::new(buf);
    scheduling::wait_until(|| reader.is_done());
    reader.result()?;
    Ok(())
}

/// Synchronously reads exactly one byte.
pub fn read_byte() -> Result<u8, Error> {
    let mut byte = 0;
    read_any(core::slice::from_mut(&mut byte))?;
    Ok(byte)
}

/// Writes from a buffer to USB serial.
///
/// Returns how many bytes were written (and thus read from the buffer). This function does not
/// block, so if the serial is not ready for write, zero is returned.
pub fn write(buf: &[u8]) -> Result<usize, Error> {
    let params = api::write::Params { ptr: buf.as_ptr(), len: buf.len() };
    let api::write::Results { len } = unsafe { api::write(params) };
    convert(len)
}

/// Writes at least one byte from a buffer to USB serial.
///
/// This function will block if necessary.
pub fn write_any(buf: &[u8]) -> Result<usize, Error> {
    let mut writer = Writer::new(buf);
    scheduling::wait_until(|| !writer.is_empty());
    writer.result()
}

/// Writes from a buffer to USB serial until everything has been written.
///
/// This function will block if necessary.
pub fn write_all(buf: &[u8]) -> Result<(), Error> {
    let mut writer = Writer::new(buf);
    scheduling::wait_until(|| writer.is_done());
    writer.result()?;
    Ok(())
}

/// Flushes the USB serial.
pub fn flush() -> Result<(), Error> {
    let api::flush::Results { res } = unsafe { api::flush() };
    convert(res).map(|_| ())
}

/// Provides asynchronous read support.
#[must_use]
pub struct Reader<'a>(Listener<'a>);

impl<'a> Reader<'a> {
    /// Asynchronously reads from USB serial into a buffer.
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Reader(Listener::new(Kind::Reader { buffer }))
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
    pub fn result(self) -> Result<usize, Error> {
        self.0.result()
    }
}

/// Provides asynchronous write support.
#[must_use]
pub struct Writer<'a>(Listener<'a>);

impl<'a> Writer<'a> {
    /// Asynchronously writes from a buffer to USB serial.
    pub fn new(buffer: &'a [u8]) -> Self {
        Writer(Listener::new(Kind::Writer { buffer }))
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
    pub fn result(self) -> Result<usize, Error> {
        self.0.result()
    }
}

struct Listener<'a> {
    kind: Kind<'a>,
    // Whether the callback triggered since last operation.
    ready: &'static Cell<bool>,
    // The callback is registered as long as not done.
    result: Result<usize, Error>,
}

impl<'a> Listener<'a> {
    fn new(kind: Kind<'a>) -> Self {
        let ready = Box::leak(Box::new(Cell::new(true)));
        let mut listener = Listener { kind, ready, result: Ok(0) };
        if listener.is_registered() {
            let event = listener.kind.event() as usize;
            let handler_func = Self::call;
            let handler_data = ready.as_ptr() as *mut u8;
            unsafe { api::register(api::register::Params { event, handler_func, handler_data }) };
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

    fn result(mut self) -> Result<usize, Error> {
        self.update()
    }

    fn update(&mut self) -> Result<usize, Error> {
        if !self.is_registered() || !self.ready.replace(false) {
            return self.result;
        }
        let pos = self.result.as_mut().unwrap();
        match self.kind.update(*pos) {
            Ok(len) => *pos += len,
            err => self.result = err,
        }
        if !self.is_registered() {
            self.unregister();
        }
        self.result
    }

    fn is_registered(&self) -> bool {
        matches!(self.result, Ok(len) if len < self.kind.len())
    }

    fn unregister(&self) {
        let event = self.kind.event() as usize;
        unsafe { api::unregister(api::unregister::Params { event }) };
    }

    extern "C" fn call(data: *mut u8) {
        let ready = unsafe { &*(data as *mut Cell<bool>) };
        ready.set(true);
    }
}

impl<'a> Drop for Listener<'a> {
    fn drop(&mut self) {
        if self.is_registered() {
            self.unregister();
        }
        unsafe { Box::from_raw(self.ready.as_ptr()) };
    }
}

enum Kind<'a> {
    Reader { buffer: &'a mut [u8] },
    Writer { buffer: &'a [u8] },
}

impl<'a> Kind<'a> {
    fn event(&self) -> api::Event {
        match self {
            Kind::Reader { .. } => api::Event::Read,
            Kind::Writer { .. } => api::Event::Write,
        }
    }

    fn len(&self) -> usize {
        match self {
            Kind::Reader { buffer } => buffer.len(),
            Kind::Writer { buffer } => buffer.len(),
        }
    }

    fn update(&mut self, pos: usize) -> Result<usize, Error> {
        match self {
            Kind::Reader { buffer } => read(&mut buffer[pos ..]),
            Kind::Writer { buffer } => write(&buffer[pos ..]),
        }
    }
}
