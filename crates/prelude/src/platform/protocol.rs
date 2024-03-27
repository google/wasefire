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

//! Provides the applet API of the platform protocol.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cell::RefCell;

use wasefire_applet_api::platform::protocol as api;

use crate::{convert_bool, convert_unit};

/// RPC service processing function.
pub trait Service: 'static {
    /// Processes a request and returns its response.
    fn process(&mut self, request: Vec<u8>) -> Vec<u8>;
}

impl<F: FnMut(Vec<u8>) -> Vec<u8> + 'static> Service for F {
    fn process(&mut self, request: Vec<u8>) -> Vec<u8> {
        self(request)
    }
}

/// Listens on RPC requests processing them according to a service.
#[must_use]
pub struct Listener<S: Service> {
    handler: *const RefCell<S>,
}

impl<S: Service> Listener<S> {
    /// Starts listening for RPC requests.
    ///
    /// The `service` argument processes requests. Note that it may be an `Fn(request: Vec<u8>) ->
    /// Vec<u8>` closure, see [Service::process()] for callback documentation.
    ///
    /// The listener stops listening when dropped.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// Listener::new(|request| Response::from(request))
    /// ```
    pub fn new(service: S) -> Self {
        let handler_func = Self::call;
        let handler = Box::into_raw(Box::new(RefCell::new(service)));
        let handler_data = handler as *const u8;
        let params = api::register::Params { handler_func, handler_data };
        convert_unit(unsafe { api::register(params) }).unwrap();
        Listener { handler }
    }

    /// Stops listening.
    ///
    /// This is equivalent to calling `core::mem::drop()`.
    pub fn stop(self) {
        core::mem::drop(self);
    }

    /// Drops the listener but continues listening.
    ///
    /// This is equivalent to calling `core::mem::forget()`. This can be useful if the listener is
    /// created deeply in the stack but the callback must continue processing events until the
    /// applet exits or traps.
    pub fn leak(self) {
        core::mem::forget(self);
    }

    extern "C" fn call(data: *const u8) {
        let handler = unsafe { &*(data as *const RefCell<S>) };
        let mut ptr = core::ptr::null_mut();
        let mut len = 0;
        let params = api::read::Params { ptr: &mut ptr, len: &mut len };
        if !convert_bool(unsafe { api::read(params) }).unwrap() {
            return; // spurious call
        }
        let request = unsafe { Vec::from_raw_parts(ptr, len, len) };
        // The borrow_mut panics if we receive another request before responding to this one. This
        // only happens if the platform has a bug.
        let response = handler.borrow_mut().process(request);
        let params = api::write::Params { ptr: response.as_ptr(), len: response.len() };
        convert_unit(unsafe { api::write(params) }).unwrap();
    }
}

impl<S: Service> Drop for Listener<S> {
    fn drop(&mut self) {
        convert_unit(unsafe { api::unregister() }).unwrap();
        drop(unsafe { Box::from_raw(self.handler as *mut RefCell<S>) });
    }
}
