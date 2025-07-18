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
use wasefire_board_api::crypto::ed25519::Api;
use wasefire_error::{Code, Error};

use crate::board::crypto::{memshred, try_from_bytes, try_from_bytes_mut};
use crate::crypto::common::{BlindedKey, EccKeyMode, KeyMode, UnblindedKey};
use crate::crypto::ed25519;

pub enum Impl {}

impl Supported for Impl {}

impl Api for Impl {
    const PRIVATE: Layout = Layout::new::<Private>();
    const PUBLIC: Layout = Layout::new::<Public>();
    // TODO: We should use key wrapping once we figure out which key encryption key to use.
    const WRAPPED: usize = core::mem::size_of::<Private>();

    fn generate(private: &mut [u8]) -> Result<(), Error> {
        let private = try_from_bytes_mut::<Private>(private)?;
        let (privkey, pubkey) = ed25519::keygen()?;
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

    fn sign(
        private: &[u8], message: &[u8], r: &mut [u8; 32], s: &mut [u8; 32],
    ) -> Result<(), Error> {
        let private = try_from_bytes::<Private>(private)?;
        let mut signature = [0; 16];
        let private = unsafe { private.borrow() }?;
        ed25519::sign(&private, message, &mut signature)?;
        let signature = bytemuck::bytes_of_mut(&mut signature);
        signature.reverse();
        s.copy_from_slice(&signature[.. 32]);
        r.copy_from_slice(&signature[32 ..]);
        Ok(())
    }

    fn verify(public: &[u8], message: &[u8], r: &[u8; 32], s: &[u8; 32]) -> Result<bool, Error> {
        let public = try_from_bytes::<Public>(public)?;
        let mut signature = [0; 16];
        let sig = bytemuck::bytes_of_mut(&mut signature);
        sig[.. 32].copy_from_slice(s);
        sig[32 ..].copy_from_slice(r);
        sig.reverse();
        let public = unsafe { public.borrow() }?;
        ed25519::verify(&public, message, &signature)
    }

    fn drop_private(private: &mut [u8]) -> Result<(), Error> {
        let private = try_from_bytes_mut::<Private>(private)?;
        memshred(&mut private.keyblob);
        Ok(())
    }

    fn export_private(private: &[u8], wrapped: &mut [u8]) -> Result<(), Error> {
        if private.len() != wrapped.len() {
            return Err(Error::user(Code::InvalidLength));
        }
        wrapped.copy_from_slice(private);
        Ok(())
    }

    fn import_private(wrapped: &[u8], private: &mut [u8]) -> Result<(), Error> {
        if private.len() != wrapped.len() {
            return Err(Error::user(Code::InvalidLength));
        }
        private.copy_from_slice(wrapped);
        Ok(())
    }

    fn export_public(public: &[u8], a: &mut [u8; 32]) -> Result<(), Error> {
        let public = try_from_bytes::<Public>(public)?;
        let key = bytemuck::bytes_of(&public.key);
        a.copy_from_slice(key);
        a.reverse();
        Ok(())
    }

    fn import_public(a: &[u8; 32], public: &mut [u8]) -> Result<(), Error> {
        let public = try_from_bytes_mut::<Public>(public)?;
        let key = bytemuck::bytes_of_mut(&mut public.key);
        key.copy_from_slice(a);
        key.reverse();
        public.checksum = 0;
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

impl Public {
    unsafe fn borrow(&self) -> Result<UnblindedKey, Error> {
        unsafe { UnblindedKey::new(KeyMode::Ecc(EccKeyMode::Ed25519), &self.key, self.checksum) }
    }
}

impl Private {
    unsafe fn borrow(&self) -> Result<BlindedKey, Error> {
        unsafe { BlindedKey::new(KeyMode::Ecc(EccKeyMode::Ed25519), &self.keyblob, self.checksum) }
    }
}
