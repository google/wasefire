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

// sw/device/lib/crypto/include/drbg.h

use wasefire_error::Error;

use crate::crypto::common::{ConstByteBuf, Word32Buf};
use crate::error::unwrap_status;

pub fn instantiate() -> Result<(), Error> {
    unwrap_status(unsafe { otcrypto_drbg_instantiate(&mut b"wasefire"[..].into()) })?;
    Ok(())
}

pub fn generate_bytes(output: &mut [u8]) -> Result<(), Error> {
    let (prefix, body, suffix) = bytemuck::pod_align_to_mut(output);
    generate_words(body)?;
    let len = prefix.len() + suffix.len();
    if 0 < len {
        let mut extra = [0u32; 2];
        let extra = &mut extra[.. len.div_ceil(4)];
        generate_words(extra)?;
        let extra = &bytemuck::cast_slice::<_, u8>(extra)[.. len];
        prefix.copy_from_slice(&extra[.. prefix.len()]);
        suffix.copy_from_slice(&extra[prefix.len() ..]);
    }
    Ok(())
}

pub fn generate_words(output: &mut [u32]) -> Result<(), Error> {
    let mut salt: ConstByteBuf = (b"" as &'static [u8]).into();
    unwrap_status(unsafe { otcrypto_drbg_generate(&mut salt, &mut output.into()) })?;
    Ok(())
}

unsafe extern "C" {
    fn otcrypto_drbg_instantiate(perso: *mut ConstByteBuf) -> i32;
    fn otcrypto_drbg_generate(salt: *mut ConstByteBuf, output: *mut Word32Buf) -> i32;
}
