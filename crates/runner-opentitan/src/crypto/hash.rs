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

// sw/device/lib/crypto/include/hash.h

use wasefire_error::Error;

use crate::crypto::common::{ConstByteBuf, HashDigest, HashMode};
use crate::error::unwrap_status;

// otcrypto_hash_context_t
pub struct Context {
    mode: HashMode,
    data: [u32; 92],
}

impl Context {
    pub fn init(mode: HashMode) -> Result<Self, Error> {
        let mut context = Context { mode, data: [0; _] };
        let status = unsafe { otcrypto_hash_init(context.to_c(), mode.to_c()) };
        unwrap_status(status)?;
        Ok(context)
    }

    pub fn update(&mut self, data: &[u8]) -> Result<(), Error> {
        let status = unsafe { otcrypto_hash_update(self.to_c(), data.into()) };
        unwrap_status(status)?;
        Ok(())
    }

    pub fn finalize(mut self, digest: &mut [u32]) -> Result<(), Error> {
        let digest =
            HashDigest { mode: self.mode.to_c(), data: digest.as_mut_ptr(), len: digest.len() };
        let status = unsafe { otcrypto_hash_final(self.to_c(), digest) };
        unwrap_status(status)?;
        Ok(())
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_c(&mut self) -> *mut u32 {
        self.data.as_mut_ptr()
    }
}

unsafe extern "C" {
    fn otcrypto_hash_init(ctx: *mut u32, mode: i32) -> i32;
    fn otcrypto_hash_update(ctx: *mut u32, message: ConstByteBuf) -> i32;
    fn otcrypto_hash_final(ctx: *mut u32, digest: HashDigest) -> i32;
}
