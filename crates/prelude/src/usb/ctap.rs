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

//! Provides API for CTAP HID.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cell::Cell;
use core::fmt::Debug;

use wasefire_applet_api::usb::serial as api;

use crate::{convert, convert_unit, scheduling, Error};

/// Implements the [`CtapHid`](crate::ctap::CtapHid) interface for USB transport.
pub struct CtapHid;

impl CtapHid {
    /// Reads from the CtapHid into a buffer without blocking.
    ///
    /// Returns how many bytes were read (and thus written to the buffer). This function does not
    /// block, so if there are no data available for read, zero is returned.
    pub fn read(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        let params = api::read::Params { ptr: buffer.as_mut_ptr(), len: buffer.len() };
        convert(unsafe { api::read(params) })
    }

    /// Synchronously reads at least one byte from a CtapHid into a buffer.
    ///
    /// This function will block if necessary.
    pub fn read_any(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        let mut reader = Reader::new(self, buffer);
        scheduling::wait_until(|| !reader.is_empty());
        reader.result()
    }

    /// Synchronously reads from a CtapHid into a buffer until it is filled.
    ///
    /// This function will block if necessary.
    pub fn read_all(&self, buffer: &mut [u8]) -> Result<(), Error> {
        let mut reader = Reader::new(self, buffer);
        scheduling::wait_until(|| reader.is_done());
        reader.result()?;
        Ok(())
    }

    /// Synchronously reads exactly one byte.
    pub fn read_byte(&self) -> Result<u8, Error> {
        let mut byte = 0;
        self.read_any(core::slice::from_mut(&mut byte))?;
        Ok(byte)
    }

    /// Writes from a buffer to the CtapHid.
    ///
    /// Returns how many bytes were written (and thus read from the buffer). This function does not
    /// block, so if the CtapHid is not ready for write, zero is returned.
    pub fn write(&self, buffer: &[u8]) -> Result<usize, Error> {
        let params = api::write::Params { ptr: buffer.as_ptr(), len: buffer.len() };
        convert(unsafe { api::write(params) })
    }

    /// Writes at least one byte from a buffer to a CtapHid.
    ///
    /// This function will block if necessary.
    pub fn write_any(&self, buffer: &[u8]) -> Result<usize, Error> {
        let mut writer = Writer::new(self, buffer);
        scheduling::wait_until(|| !writer.is_empty());
        writer.result()
    }

    /// Writes from a buffer to a CtapHid until everything has been written.
    ///
    /// This function will block if necessary.
    pub fn write_all(&self, buffer: &[u8]) -> Result<(), Error> {
        let mut writer = Writer::new(self, buffer);
        scheduling::wait_until(|| writer.is_done());
        writer.result()?;
        Ok(())
    }

    /// Registers a callback for an event.
    ///
    /// # Safety
    ///
    /// The function pointer and data must live until unregistered. The function must support
    /// concurrent calls.
    unsafe fn register(
        &self, event: Event, func: extern "C" fn(*const u8), data: *const u8,
    ) -> Result<(), Error> {
        let params = api::register::Params {
            event: convert_event(event) as usize,
            handler_func: func,
            handler_data: data,
        };
        convert_unit(unsafe { api::register(params) })
    }

    /// Unregisters the callback for an event.
    fn unregister(&self, event: Event) -> Result<(), Error> {
        let params = api::unregister::Params { event: convert_event(event) as usize };
        convert_unit(unsafe { api::unregister(params) })
    }
}

/// CtapHid events to be notified.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Event {
    /// The CtapHid may be ready to read.
    Read,

    /// The CtapHid may be ready to write.
    Write,
}

fn convert_event(event: Event) -> api::Event {
    match event {
        Event::Read => api::Event::Read,
        Event::Write => api::Event::Write,
    }
}

/// Asynchronously listens for event notifications.
pub struct Listener<'a> {
    ctap_hid: &'a CtapHid,
    event: Event,
    notified: &'static Cell<bool>,
}

impl<'a> Listener<'a> {
    /// Starts listening for the provided event until dropped.
    pub fn new(ctap_hid: &'a CtapHid, event: Event) -> Self {
        let notified = Box::leak(Box::new(Cell::new(true)));
        let func = Self::call;
        let data = notified as *mut _ as *const u8;
        unsafe { ctap_hid.register(event, func, data) }.unwrap();
        Listener { ctap_hid, event, notified }
    }

    /// Returns whether the event triggered since the last call.
    pub fn is_notified(&mut self) -> bool {
        self.notified.replace(false)
    }

    extern "C" fn call(data: *const u8) {
        let notified = unsafe { &*(data as *const Cell<bool>) };
        notified.set(true);
    }
}

impl<'a> Drop for Listener<'a> {
    fn drop(&mut self) {
        self.ctap_hid.unregister(self.event).unwrap();
        drop(unsafe { Box::from_raw(self.notified.as_ptr()) });
    }
}

/// Asynchronously reads delimited frames.
///
/// If you want to read at most a given amount instead, use [`Reader`].
pub struct DelimitedReader<'a> {
    listener: Listener<'a>,
    buffer: Vec<u8>,
    frame: Option<usize>, // index of first delimiter in buffer, if any
    delimiter: u8,
}

impl<'a> DelimitedReader<'a> {
    /// Starts reading delimited frames from a CtapHid.
    pub fn new(ctap_hid: &'a CtapHid, delimiter: u8) -> Self {
        let listener = Listener::new(ctap_hid, Event::Read);
        DelimitedReader { listener, buffer: Vec::new(), frame: None, delimiter }
    }

    /// Returns the next delimited frame (including the delimiter), if any.
    ///
    /// This function should be called until it returns `None` before waiting for callback again.
    /// Otherwise, it may be possible that the platform doesn't notify for new data if the existing
    /// data has not been read.
    pub fn next_frame(&mut self) -> Option<Vec<u8>> {
        if self.frame.is_none() && self.listener.is_notified() {
            self.flush();
        }
        self.frame.map(|len| {
            let mut frame = self.buffer.split_off(len + 1);
            core::mem::swap(&mut frame, &mut self.buffer);
            self.frame = self.buffer.iter().position(|&x| x == self.delimiter);
            frame
        })
    }

    /// Stops reading and returns the current buffer.
    ///
    /// The buffer may contain multiple delimited frames.
    pub fn stop(self) -> Vec<u8> {
        self.buffer
    }

    fn flush(&mut self) {
        while self.read() {}
    }

    fn read(&mut self) -> bool {
        let mut data = [0; 32];
        let len = self.listener.ctap_hid.read(&mut data).unwrap();
        let pos = self.buffer.len();
        self.buffer.extend_from_slice(&data[.. len]);
        if self.frame.is_none() {
            for i in pos .. pos + len {
                if self.buffer[i] == self.delimiter {
                    self.frame = Some(i);
                    break;
                }
            }
        }
        len == data.len()
    }
}

/// Asynchronously reads into the provided buffer.
///
/// If instead you want to continuously read delimited frames, use [`DelimitedReader`].
#[must_use]
pub struct Reader<'a>(Updater<'a>);

impl<'a> Reader<'a> {
    /// Asynchronously reads from a CtapHid into a buffer.
    pub fn new(ctap_hid: &'a CtapHid, buffer: &'a mut [u8]) -> Self {
        Reader(Updater::new(ctap_hid, Kind::Reader { buffer }))
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

/// Asynchronously writes from the provided buffer.
#[must_use]
pub struct Writer<'a>(Updater<'a>);

impl<'a> Writer<'a> {
    /// Asynchronously writes from a buffer to a CtapHid.
    pub fn new(ctap_hid: &'a CtapHid, buffer: &'a [u8]) -> Self {
        Writer(Updater::new(ctap_hid, Kind::Writer { buffer }))
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

struct Updater<'a> {
    // The listener is alive as long as not done (see `should_listen()`).
    listener: Option<Listener<'a>>,
    kind: Kind<'a>,
    result: Result<usize, Error>,
}

impl<'a> Updater<'a> {
    fn new(ctap_hid: &'a CtapHid, kind: Kind<'a>) -> Self {
        let event = kind.event();
        let mut result = Updater { listener: None, kind, result: Ok(0) };
        if result.should_listen() {
            result.listener = Some(Listener::new(ctap_hid, event));
        }
        let _ = result.update();
        result
    }

    fn is_empty(&mut self) -> bool {
        matches!(self.update(), Ok(0))
    }

    fn is_done(&mut self) -> bool {
        let _ = self.update();
        self.listener.is_none()
    }

    fn result(mut self) -> Result<usize, Error> {
        self.update()
    }

    fn update(&mut self) -> Result<usize, Error> {
        let listener = match &mut self.listener {
            Some(x) => x,
            None => return self.result,
        };
        if !listener.is_notified() {
            return self.result;
        }
        let pos = self.result.as_mut().unwrap();
        match self.kind.update(listener.ctap_hid, *pos) {
            Ok(len) => *pos += len,
            err => self.result = err,
        }
        if !self.should_listen() {
            self.listener = None;
        }
        self.result
    }

    fn should_listen(&self) -> bool {
        matches!(self.result, Ok(len) if len < self.kind.len())
    }
}

enum Kind<'a> {
    Reader { buffer: &'a mut [u8] },
    Writer { buffer: &'a [u8] },
}

impl<'a> Kind<'a> {
    fn event(&self) -> Event {
        match self {
            Kind::Reader { .. } => Event::Read,
            Kind::Writer { .. } => Event::Write,
        }
    }

    fn len(&self) -> usize {
        match self {
            Kind::Reader { buffer } => buffer.len(),
            Kind::Writer { buffer } => buffer.len(),
        }
    }

    fn update(&mut self, ctap_hid: &CtapHid, pos: usize) -> Result<usize, Error> {
        match self {
            Kind::Reader { buffer } => ctap_hid.read(&mut buffer[pos ..]),
            Kind::Writer { buffer } => ctap_hid.write(&buffer[pos ..]),
        }
    }
}
