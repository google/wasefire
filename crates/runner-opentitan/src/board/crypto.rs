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

use wasefire_board_api::crypto;
use wasefire_error::{Code, Error};

mod aes;
mod ecdh;
mod ecdsa;
#[cfg(feature = "ed25519")]
mod ed25519;
mod hmac;

pub enum Impl {}

impl crypto::Api for Impl {
    type Aes256Cbc = aes::Impl;
    #[cfg(feature = "ed25519")]
    type Ed25519 = ed25519::Impl;
    #[cfg(feature = "software-ed25519")]
    type Ed25519 = crypto::SoftwareEd25519<crypto::CryptoRng<super::Board>>;
    type HmacSha256 = hmac::HmacSha256;
    type P256Ecdh = ecdh::Impl;
    type P256Ecdsa = ecdsa::Impl;
    type Sha256 = hmac::Sha256;
}

fn try_from_bytes<T: bytemuck::Pod>(bytes: &[u8]) -> Result<&T, Error> {
    bytemuck::try_from_bytes(bytes).map_err(|_| Error::user(Code::InvalidArgument))
}

fn try_from_bytes_mut<T: bytemuck::Pod>(bytes: &mut [u8]) -> Result<&mut T, Error> {
    bytemuck::try_from_bytes_mut(bytes).map_err(|_| Error::user(Code::InvalidArgument))
}

fn memshred(buffer: &mut [u32]) {
    unsafe { hardened_memshred(buffer.as_mut_ptr(), buffer.len()) }
}

unsafe extern "C" {
    fn hardened_memshred(dest: *mut u32, word_len: usize);
}
