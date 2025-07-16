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

use core::alloc::Layout;

use wasefire_board_api::Supported;
use wasefire_board_api::crypto::ecdh::Api;
use wasefire_error::Error;

use crate::board::crypto::{memshred, try_from_bytes, try_from_bytes_mut};
use crate::crypto::common::{BlindedKey, KeyMode, UnblindedKey};
use crate::crypto::p256;

pub enum Impl {}

impl Supported for Impl {}

impl Api<32> for Impl {
    const PRIVATE: Layout = Layout::new::<Private>();
    const PUBLIC: Layout = Layout::new::<Public>();
    const SHARED: Layout = Layout::new::<Shared>();

    fn generate(private: &mut [u8]) -> Result<(), Error> {
        let private = try_from_bytes_mut::<Private>(private)?;
        let (privkey, pubkey) = p256::ecdh_keygen()?;
        private.keyblob.copy_from_slice(privkey.0.keyblob());
        private.checksum = privkey.0.checksum;
        private.public.key.copy_from_slice(pubkey.0.key());
        private.public.checksum = pubkey.0.checksum;
        Ok(())
    }

    fn public(private: &[u8], public: &mut [u8]) -> Result<(), Error> {
        let private = try_from_bytes::<Private>(private)?;
        let public = try_from_bytes_mut::<Public>(public)?;
        *public = private.public;
        Ok(())
    }

    fn shared(private: &[u8], public: &[u8], shared: &mut [u8]) -> Result<(), Error> {
        let private = try_from_bytes::<Private>(private)?;
        let public = try_from_bytes::<Public>(public)?;
        let shared = try_from_bytes_mut::<Shared>(shared)?;
        let private = unsafe { private.borrow() }?;
        let public = unsafe { public.borrow() }?;
        let secret = p256::ecdh(&private, &public, KeyMode::AesCbc)?;
        shared.keyblob.copy_from_slice(secret.0.keyblob());
        shared.checksum = secret.0.checksum;
        Ok(())
    }

    fn drop_private(private: &mut [u8]) -> Result<(), Error> {
        let private = try_from_bytes_mut::<Private>(private)?;
        memshred(&mut private.keyblob);
        Ok(())
    }

    fn drop_shared(shared: &mut [u8]) -> Result<(), Error> {
        let shared = try_from_bytes_mut::<Shared>(shared)?;
        memshred(&mut shared.keyblob);
        Ok(())
    }

    fn export_public(public: &[u8], x: &mut [u8; 32], y: &mut [u8; 32]) -> Result<(), Error> {
        let public = try_from_bytes::<Public>(public)?;
        let key = bytemuck::bytes_of(&public.key);
        x.copy_from_slice(&key[.. 32]);
        y.copy_from_slice(&key[32 ..]);
        x.reverse();
        y.reverse();
        Ok(())
    }

    fn import_public(x: &[u8; 32], y: &[u8; 32], public: &mut [u8]) -> Result<(), Error> {
        let public = try_from_bytes_mut::<Public>(public)?;
        let key = bytemuck::bytes_of_mut(&mut public.key);
        key[.. 32].copy_from_slice(y);
        key[32 ..].copy_from_slice(x);
        key.reverse();
        public.checksum = 0;
        Ok(())
    }

    fn export_shared(shared: &[u8], x: &mut [u8; 32]) -> Result<(), Error> {
        let shared = try_from_bytes::<Shared>(shared)?;
        let keyblob = bytemuck::bytes_of(&shared.keyblob);
        let share0 = &keyblob[.. 32];
        let share1 = &keyblob[32 ..];
        for (x, (s0, s1)) in x.iter_mut().zip(share0.iter().zip(share1.iter())) {
            *x = s0 ^ s1;
        }
        x.reverse();
        Ok(())
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Public {
    key: [u32; 16],
    checksum: u32,
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Private {
    keyblob: [u32; 20],
    checksum: u32,
    // TODO: Remove once cryptolib provides a function to recover the public key of a private key.
    public: Public,
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Shared {
    keyblob: [u32; 16],
    checksum: u32,
}

impl Public {
    unsafe fn borrow(&self) -> Result<UnblindedKey, Error> {
        unsafe { UnblindedKey::new(KeyMode::EcdhP256, &self.key, self.checksum) }
    }
}

impl Private {
    unsafe fn borrow(&self) -> Result<BlindedKey, Error> {
        unsafe { BlindedKey::new(KeyMode::EcdhP256, &self.keyblob, self.checksum) }
    }
}
