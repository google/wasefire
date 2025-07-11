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

use wasefire_error::Error;

use crate::crypto::common::{BlindedKey, ConstByteBuf, OwnedBlindedKey, Word32Buf};
use crate::error::unwrap_status;

// sw/device/lib/crypto/include/hmac.h:otcrypto_hmac_context_t
#[repr(C)]
pub struct Context {
    key: OwnedBlindedKey,
    data: [u32; 92],
}

impl Context {
    pub fn init(key: OwnedBlindedKey) -> Result<Self, Error> {
        let mut context = Context { key, data: [0; _] };
        let status = unsafe { otcrypto_hmac_init(context.to_c(), &context.key.0) };
        unwrap_status(status)?;
        Ok(context)
    }

    pub fn update(&mut self, data: &[u8]) -> Result<(), Error> {
        let status = unsafe { otcrypto_hmac_update(self.to_c(), data.into()) };
        unwrap_status(status)?;
        Ok(())
    }

    pub fn finalize(mut self, tag: &mut [u32]) -> Result<(), Error> {
        let status = unsafe { otcrypto_hmac_final(self.to_c(), tag.into()) };
        unwrap_status(status)?;
        Ok(())
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_c(&mut self) -> *mut u32 {
        self.data.as_mut_ptr()
    }
}

unsafe extern "C" {
    fn otcrypto_hmac_init(ctx: *mut u32, key: *const BlindedKey) -> i32;
    fn otcrypto_hmac_update(ctx: *mut u32, message: ConstByteBuf) -> i32;
    fn otcrypto_hmac_final(ctx: *mut u32, tag: Word32Buf) -> i32;
}
