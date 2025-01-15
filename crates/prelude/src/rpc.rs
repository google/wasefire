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

//! Provides a generic API for RPCs.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cell::RefCell;

use sealed::sealed;
use wasefire_error::Error;

/// Provides high-level RPC API from low-level API.
///
/// This trait should only be implemented by the prelude and is thus sealed. Its purpose is to
/// provide a unique interface to the different RPC implementations.
#[sealed(pub(crate))]
pub trait Rpc {
    /// Reads the last request, if any.
    ///
    /// This function does not block, so if no request was received, `None` is returned.
    fn read(&self) -> Result<Option<Vec<u8>>, Error>;

    /// Writes a response to the last request.
    ///
    /// This function does not block. The response is always buffered until sent.
    fn write(&self, response: &[u8]) -> Result<(), Error>;

    /// Registers a callback when a request is received.
    ///
    /// # Safety
    ///
    /// The function pointer and data must live until unregistered. The function must support
    /// concurrent calls.
    unsafe fn register(&self, func: extern "C" fn(*const u8), data: *const u8)
    -> Result<(), Error>;

    /// Unregisters the callback.
    fn unregister(&self) -> Result<(), Error>;
}

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
pub struct Listener<'a, T: Rpc, S: Service> {
    // Safety: This is a `Box<State<'a, T, S>>` we own and lend the platform as a shared borrow
    // with a lifetime starting at `Rpc::register()` and ending at `Rpc::unregister()`.
    state: *const State<'a, T, S>,
}

struct State<'a, T: Rpc, S: Service> {
    rpc: &'a T,
    handler: RefCell<S>,
}

impl<'a, T: Rpc, S: Service> Listener<'a, T, S> {
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
    /// Listener::new(&platform::protocol::RpcProtocol, |request| Response::from(request))
    /// ```
    pub fn new(rpc: &'a T, service: S) -> Self {
        let state = Box::into_raw(Box::new(State { rpc, handler: RefCell::new(service) }));
        // SAFETY: The `state` lifetime ends after unregister (see field invariant).
        unsafe { rpc.register(Self::call, state as *const u8) }.unwrap();
        Listener { state }
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
    /// created deeply in the stack but the callback must continue processing RPCs until the applet
    /// exits or traps.
    pub fn leak(self) {
        core::mem::forget(self);
    }

    extern "C" fn call(state: *const u8) {
        // SAFETY: `state` is the shared borrow we lent the platform (see field invariant). We are
        // borrowing it back for the duration of this call.
        let state = unsafe { &*(state as *const State<'a, T, S>) };
        let request = match state.rpc.read().unwrap() {
            Some(x) => x,
            None => return, // spurious call
        };
        // The borrow_mut panics if we receive another request before responding to this one. This
        // only happens if the platform has a bug.
        let response = state.handler.borrow_mut().process(request);
        state.rpc.write(&response).unwrap();
    }
}

impl<'a, T: Rpc, S: Service> Drop for Listener<'a, T, S> {
    fn drop(&mut self) {
        let state = unsafe { &*self.state };
        state.rpc.unregister().unwrap();
        // SAFETY: `self.state` is a `Box<State<'a, T, S>>` we own back, now that the lifetime of
        // the platform borrow is over (see field invariant).
        drop(unsafe { Box::from_raw(self.state as *mut State<'a, T, S>) });
    }
}
