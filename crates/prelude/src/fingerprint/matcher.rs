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

//! Provides fingerprint matcher.

use alloc::boxed::Box;
use alloc::vec::Vec;

use wasefire_applet_api::fingerprint::matcher as api;
use wasefire_common::ptr::{OwnedPtr, SharedPtr};
use wasefire_error::Code;
use wasefire_sync::{Lazy, Mutex};

use crate::{Error, convert, convert_bool, convert_unit};

/// Whether fingerprint matcher is supported.
pub fn is_supported() -> bool {
    convert_bool(unsafe { api::is_supported() }).unwrap_or(false)
}

fn template_length() -> usize {
    static TEMPLATE_LENGTH: Lazy<usize> =
        Lazy::new(|| convert(unsafe { api::template_length() }).unwrap() as usize);
    *TEMPLATE_LENGTH
}

/// State of the enrollment of a new finger.
pub struct Enroll {
    // Safety: This is a `Box<Mutex<EnrollState>>` we own and lend the platform as a shared
    // borrow between the call to `api::enroll()` and either `Self::abort()` or `Self::done()`.
    state: SharedPtr<Mutex<EnrollState>>,
    // Whether the state has already been dropped.
    dropped: bool,
}

impl Drop for Enroll {
    fn drop(&mut self) {
        if !self.dropped {
            self.drop_state().unwrap();
        }
    }
}

/// Current progress of an enrollment.
#[derive(Clone, Copy)]
pub struct EnrollProgress {
    /// Number of detected touches so far.
    pub detected: usize,
    /// Estimated number of remaining touches.
    pub remaining: Option<usize>,
}

enum EnrollState {
    Running(EnrollProgress),
    Done { template: OwnedPtr<u8> },
    Failed { error: Error },
}

const INIT_ENROLL_STATE: EnrollState =
    EnrollState::Running(EnrollProgress { detected: 0, remaining: None });

impl Enroll {
    /// Starts the enrollment of a new finger.
    pub fn new() -> Result<Self, Error> {
        let handler_step_func = Self::step;
        let handler_func = Self::done;
        let state = Box::into_raw(Box::new(Mutex::new(INIT_ENROLL_STATE)));
        let handler_step_data = state as *const u8;
        let handler_data = state as *const u8;
        let params = api::enroll::Params {
            handler_step_func,
            handler_step_data,
            handler_func,
            handler_data,
        };
        convert_unit(unsafe { api::enroll(params) })?;
        Ok(Enroll { state: SharedPtr(state), dropped: false })
    }

    /// Returns the enrollment progress.
    ///
    /// Returns `None` if the enrollment finished. Use `Self::result()` to access the result.
    pub fn progress(&self) -> Option<EnrollProgress> {
        match *Self::state(self.state.0 as *const u8).lock() {
            EnrollState::Running(x) => Some(x),
            EnrollState::Done { .. } | EnrollState::Failed { .. } => None,
        }
    }

    /// Returns the result of the enrollment process.
    ///
    /// In case of success, the template ID of the enrolled finger is returned.
    pub fn result(mut self) -> Result<Box<[u8]>, Error> {
        match Self::into_state(&mut self) {
            EnrollState::Running(_) => {
                let _ = self.abort();
                Err(Error::user(Code::InvalidState))
            }
            EnrollState::Done { template } => Ok(new_template(template)),
            EnrollState::Failed { error } => Err(error),
        }
    }

    /// Aborts the enrollment process.
    pub fn abort(mut self) -> Result<(), Error> {
        self.drop_state()
    }

    fn drop_state(&mut self) -> Result<(), Error> {
        if matches!(Self::into_state(self), EnrollState::Running(_)) {
            convert_unit(unsafe { api::abort_enroll() })?;
        }
        Ok(())
    }

    extern "C" fn step(data: *const u8, remaining: usize) {
        let mut state = Self::state(data).lock();
        match *state {
            EnrollState::Running(ref mut progress) => {
                progress.detected += 1;
                progress.remaining = Some(remaining);
            }
            // Those should not happen, but if they do, we ignore the step.
            EnrollState::Done { .. } | EnrollState::Failed { .. } => (),
        }
    }

    extern "C" fn done(data: *const u8, result: isize, template: *mut u8) {
        let mut state = Self::state(data).lock();
        *state = match convert_unit(result) {
            Ok(()) => EnrollState::Done { template: OwnedPtr(template) },
            Err(error) => EnrollState::Failed { error },
        }
    }

    fn state<'a>(data: *const u8) -> &'a Mutex<EnrollState> {
        // SAFETY: `data` is the `&EnrollState` we lent the platform (see `Enroll::state`). We are
        // borrowing it back for the duration of this call.
        unsafe { &*(data as *const Mutex<EnrollState>) }
    }

    #[allow(clippy::wrong_self_convention)]
    fn into_state(&mut self) -> EnrollState {
        assert!(!self.dropped);
        self.dropped = true;
        // SAFETY: `self.state` is a `Box<Mutex<EnrollState>>` we own back, now that the lifetime
        // of the platform borrow is over (see `Enroll::state`).
        unsafe { Box::from_raw(self.state.0 as *mut Mutex<EnrollState>) }.into_inner()
    }
}

/// State of the identification of a finger.
pub struct Identify {
    // Safety: This is a `Box<Mutex<IdentifyState>>` we own and lend the platform as a shared
    // borrow between the call to `api::identify()` and either `Self::abort()` or `Self::call()`.
    state: SharedPtr<Mutex<IdentifyState>>,
    // Whether the state has already been dropped.
    dropped: bool,
}

impl Drop for Identify {
    fn drop(&mut self) {
        if !self.dropped {
            self.drop_state().unwrap();
        }
    }
}

enum IdentifyState {
    Started,
    Done { matched: bool, template: OwnedPtr<u8> },
    Failed { error: Error },
}

/// Result of an identification.
pub enum IdentifyResult {
    /// The finger did not match any template (or the provided template).
    NoMatch,
    /// The finger matched one of the templates.
    Match {
        /// The template ID that was matched.
        template: Box<[u8]>,
    },
}

impl Identify {
    /// Starts the identification of a finger.
    pub fn new(template: Option<&[u8]>) -> Result<Self, Error> {
        let template = opt_template(template)?;
        let handler_func = Self::call;
        let state = Box::into_raw(Box::new(Mutex::new(IdentifyState::Started)));
        let handler_data = state as *const u8;
        let params = api::identify::Params { template, handler_func, handler_data };
        convert_unit(unsafe { api::identify(params) })?;
        Ok(Identify { state: SharedPtr(state), dropped: false })
    }

    /// Returns whether the identification has been completed.
    ///
    /// Use `Self::result()` to access the result.
    pub fn is_done(&self) -> bool {
        match *Self::state(self.state.0 as *const u8).lock() {
            IdentifyState::Started => false,
            IdentifyState::Done { .. } | IdentifyState::Failed { .. } => true,
        }
    }

    /// Returns the result of the identification process.
    pub fn result(mut self) -> Result<IdentifyResult, Error> {
        match Self::into_state(&mut self) {
            IdentifyState::Started => {
                let _ = self.abort();
                Err(Error::user(Code::InvalidState))
            }
            IdentifyState::Done { matched: false, .. } => Ok(IdentifyResult::NoMatch),
            IdentifyState::Done { matched: true, template } => {
                Ok(IdentifyResult::Match { template: new_template(template) })
            }
            IdentifyState::Failed { error } => Err(error),
        }
    }

    /// Aborts the identification process.
    pub fn abort(mut self) -> Result<(), Error> {
        self.drop_state()
    }

    fn drop_state(&mut self) -> Result<(), Error> {
        if matches!(Self::into_state(self), IdentifyState::Started) {
            convert_unit(unsafe { api::abort_identify() })?;
        }
        Ok(())
    }

    extern "C" fn call(data: *const u8, result: isize, template: *mut u8) {
        let mut state = Self::state(data).lock();
        *state = match convert_bool(result) {
            Ok(matched) => IdentifyState::Done { matched, template: OwnedPtr(template) },
            Err(error) => IdentifyState::Failed { error },
        }
    }

    fn state<'a>(data: *const u8) -> &'a Mutex<IdentifyState> {
        // SAFETY: `data` is the `&IdentifyState` we lent the platform (see `Identify::state`). We
        // are borrowing it back for the duration of this call.
        unsafe { &*(data as *const Mutex<IdentifyState>) }
    }

    #[allow(clippy::wrong_self_convention)]
    fn into_state(&mut self) -> IdentifyState {
        assert!(!self.dropped);
        self.dropped = true;
        // SAFETY: `self.state` is a `Box<Mutex<IdentifyState>>` we own back, now that the
        // lifetime of the platform borrow is over (see `Identify::state`).
        unsafe { Box::from_raw(self.state.0 as *mut Mutex<IdentifyState>) }.into_inner()
    }
}

/// Deletes all enrolled fingers (or a specific on given its template ID).
pub fn delete_template(template: Option<&[u8]>) -> Result<(), Error> {
    let template = opt_template(template)?;
    let params = api::delete_template::Params { template };
    convert_unit(unsafe { api::delete_template(params) })
}

/// Lists the template ID of all enrolled fingers.
pub fn list_templates() -> Result<Templates, Error> {
    let mut ptr = core::ptr::null_mut();
    let params = api::list_templates::Params { templates: &mut ptr };
    let count = convert(unsafe { api::list_templates(params) })?;
    let len = count * template_length();
    let data = if count == 0 { Vec::new() } else { unsafe { Vec::from_raw_parts(ptr, len, len) } };
    Ok(Templates { data })
}

/// List of template IDs.
pub struct Templates {
    data: Vec<u8>,
}

impl Templates {
    /// Returns whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of templates.
    pub fn len(&self) -> usize {
        self.data.len() / template_length()
    }

    /// Iterates over the templates.
    pub fn iter(&self) -> impl Iterator<Item = &[u8]> {
        self.data.chunks(template_length())
    }
}

fn new_template(template: OwnedPtr<u8>) -> Box<[u8]> {
    let len = template_length();
    let ptr = core::ptr::slice_from_raw_parts_mut(template.0, len);
    unsafe { Box::from_raw(ptr) }
}

fn opt_template(template: Option<&[u8]>) -> Result<*const u8, Error> {
    match template {
        None => Ok(core::ptr::null()),
        Some(x) if x.len() == template_length() => Ok(x.as_ptr()),
        Some(_) => Err(Error::user(Code::InvalidLength)),
    }
}
