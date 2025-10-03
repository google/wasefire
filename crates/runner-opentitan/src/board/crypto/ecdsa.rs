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
use wasefire_board_api::crypto::ecdsa::Api;
use wasefire_error::{Code, Error};

use crate::board::crypto::{memshred, try_from_bytes, try_from_bytes_mut};
use crate::crypto::common::{BlindedKey, EccKeyMode, KeyMode, UnblindedKey};
use crate::crypto::p256;

pub enum Impl {}

impl Supported for Impl {}

impl Api<32> for Impl {
    const PRIVATE: Layout = Layout::new::<Private>();
    const PUBLIC: Layout = Layout::new::<Public>();
    // TODO: We should use key wrapping once we figure out which key encryption key to use.
    const WRAPPED: usize = core::mem::size_of::<Private>();

    fn generate(private: &mut [u8]) -> Result<(), Error> {
        let private = try_from_bytes_mut::<Private>(private)?;
        let (privkey, pubkey) = p256::ecdsa_keygen()?;
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
        private: &[u8], digest: &[u8; 32], r: &mut [u8; 32], s: &mut [u8; 32],
    ) -> Result<(), Error> {
        let private = try_from_bytes::<Private>(private)?;
        let digest = align_digest(digest);
        let mut signature = [0; 16];
        let private = unsafe { private.borrow() }?;
        p256::ecdsa_sign(&private, &digest, &mut signature)?;
        let signature = bytemuck::bytes_of_mut(&mut signature);
        signature.reverse();
        s.copy_from_slice(&signature[.. 32]);
        r.copy_from_slice(&signature[32 ..]);
        Ok(())
    }

    fn verify(public: &[u8], digest: &[u8; 32], r: &[u8; 32], s: &[u8; 32]) -> Result<bool, Error> {
        let public = try_from_bytes::<Public>(public)?;
        let digest = align_digest(digest);
        let mut signature = [0; 16];
        let sig = bytemuck::bytes_of_mut(&mut signature);
        sig[.. 32].copy_from_slice(s);
        sig[32 ..].copy_from_slice(r);
        sig.reverse();
        let public = unsafe { public.borrow() }?;
        p256::ecdsa_verify(&public, &digest, &signature)
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
        public.checksum = unsafe { public.borrow()? }.checksum();
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
        unsafe { UnblindedKey::new(KeyMode::Ecc(EccKeyMode::EcdsaP256), &self.key, self.checksum) }
    }
}

impl Private {
    unsafe fn borrow(&self) -> Result<BlindedKey, Error> {
        unsafe {
            BlindedKey::new(KeyMode::Ecc(EccKeyMode::EcdsaP256), &self.keyblob, self.checksum)
        }
    }
}

fn align_digest(input: &[u8; 32]) -> [u32; 8] {
    let mut output = [0; 8];
    bytemuck::bytes_of_mut(&mut output).copy_from_slice(input);
    output
}
