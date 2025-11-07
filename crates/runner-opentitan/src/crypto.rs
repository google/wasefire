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

use wasefire_logger as log;

use crate::error::unwrap_status;

pub mod aes;
pub mod common;
pub mod drbg;
#[cfg(feature = "ed25519")]
pub mod ed25519;
pub mod hash;
pub mod hmac;
pub mod p256;

pub fn init() {
    let _ = unwrap_status(unsafe { entropy_complex_init() })
        .inspect_err(|e| log::error!("entropy_complex_init() failed {}", e));
}

unsafe extern "C" {
    fn entropy_complex_init() -> i32;
}

// This constant is used by cryptolib through security_config_check().
#[unsafe(no_mangle)]
static kDeviceType: i32 = 5; // kDeviceSilicon
