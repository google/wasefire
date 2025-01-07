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

use alloc::vec::Vec;

use sealed::sealed;
use wasefire_applet_api::platform::protocol as api;
use wasefire_error::Error;

use crate::{convert_bool, convert_unit};

/// Implements the [`Rpc`](crate::rpc::Rpc) interface for the platform protocol.
pub struct RpcProtocol;

#[sealed]
impl crate::rpc::Rpc for RpcProtocol {
    fn read(&self) -> Result<Option<Vec<u8>>, Error> {
        let mut ptr = core::ptr::null_mut();
        let mut len = 0;
        let params = api::read::Params { ptr: &mut ptr, len: &mut len };
        if !convert_bool(unsafe { api::read(params) })? {
            return Ok(None);
        }
        // SAFETY: If `api::read()` returns true and `len` is non-zero then it allocated. The rest
        // is similar as in `crate::platform::serial()`.
        Ok(Some(if len == 0 { Vec::new() } else { unsafe { Vec::from_raw_parts(ptr, len, len) } }))
    }

    fn write(&self, response: &[u8]) -> Result<(), Error> {
        let params = api::write::Params { ptr: response.as_ptr(), len: response.len() };
        convert_unit(unsafe { api::write(params) })
    }

    unsafe fn register(
        &self, func: extern "C" fn(*const u8), data: *const u8,
    ) -> Result<(), Error> {
        let params = api::register::Params { handler_func: func, handler_data: data };
        convert_unit(unsafe { api::register(params) })
    }

    fn unregister(&self) -> Result<(), Error> {
        convert_unit(unsafe { api::unregister() })
    }
}
