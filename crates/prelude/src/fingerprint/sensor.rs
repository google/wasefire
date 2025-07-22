// Copyright 2025 Google LLC
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

//! Provides fingerprint sensor.

use alloc::boxed::Box;
use core::cell::RefCell;

use wasefire_applet_api::fingerprint::sensor as api;
use wasefire_error::Code;

use crate::{Error, convert, convert_bool, convert_unit};

/// Whether fingerprint sensor is supported.
pub fn is_supported() -> bool {
    convert_bool(unsafe { api::is_supported() }).unwrap_or(false)
}

/// A captured image.
pub struct Image {
    /// Pixel values.
    pub data: Box<[u8]>,

    /// Width of the image.
    pub width: usize,
}

/// State of an image capture.
pub struct Capture {
    // Safety: This is a `Box<RefCell<CaptureState>>` we own and lend the platform as a shared
    // borrow between the call to `api::capture()` and either `Self::abort()` or `Self::call()`.
    state: *const u8,
    // Whether the state has already been dropped.
    dropped: bool,
}

impl Drop for Capture {
    fn drop(&mut self) {
        if !self.dropped {
            self.drop_state().unwrap();
        }
    }
}

enum CaptureState {
    Started,
    Done { image: Box<[u8]>, width: usize },
    Failed { error: Error },
}

impl Capture {
    /// Starts an image capture.
    pub fn new() -> Result<Self, Error> {
        let handler_func = Self::call;
        let state = Box::into_raw(Box::new(RefCell::new(CaptureState::Started))) as *const u8;
        let handler_data = state;
        let params = api::capture::Params { handler_func, handler_data };
        convert_unit(unsafe { api::capture(params) })?;
        Ok(Capture { state, dropped: false })
    }

    /// Returns whether the capture has completed.
    ///
    /// Use `Self::result()` to access the result.
    pub fn is_done(&self) -> bool {
        match *Self::state(self.state).borrow() {
            CaptureState::Started => false,
            CaptureState::Done { .. } | CaptureState::Failed { .. } => true,
        }
    }

    /// Returns the result of the capture.
    pub fn result(mut self) -> Result<Image, Error> {
        match Self::into_state(&mut self) {
            CaptureState::Started => {
                let _ = self.abort();
                Err(Error::user(Code::InvalidState))
            }
            CaptureState::Done { image, width } => Ok(Image { data: image, width }),
            CaptureState::Failed { error } => Err(error),
        }
    }

    /// Aborts the capture.
    pub fn abort(mut self) -> Result<(), Error> {
        self.drop_state()
    }

    fn drop_state(&mut self) -> Result<(), Error> {
        if matches!(Self::into_state(self), CaptureState::Started) {
            convert_unit(unsafe { api::abort_capture() })?;
        }
        Ok(())
    }

    extern "C" fn call(data: *const u8, ptr: *mut u8, width: isize, height: usize) {
        let mut state = Self::state(data).borrow_mut();
        *state = match convert(width) {
            Ok(width) => {
                assert!(0 < width);
                let len = width * height;
                let ptr = core::ptr::slice_from_raw_parts_mut(ptr, len);
                let image = unsafe { Box::from_raw(ptr) };
                CaptureState::Done { image, width }
            }
            Err(error) => CaptureState::Failed { error },
        }
    }

    fn state<'a>(data: *const u8) -> &'a RefCell<CaptureState> {
        // SAFETY: `data` is the `&CaptureState` we lent the platform (see `Capture::state`). We are
        // borrowing it back for the duration of this call.
        unsafe { &*(data as *const RefCell<CaptureState>) }
    }

    #[allow(clippy::wrong_self_convention)]
    fn into_state(&mut self) -> CaptureState {
        assert!(!self.dropped);
        self.dropped = true;
        // SAFETY: `self.state` is a `Box<RefCell<CaptureState>>` we own back, now that the
        // lifetime of the platform borrow is over (see `Capture::state`).
        unsafe { Box::from_raw(self.state as *mut RefCell<CaptureState>) }.into_inner()
    }
}
