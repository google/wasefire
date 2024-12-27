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

use core::mem::offset_of;

use crate::symbol_addr;

pub fn active() -> &'static Manifest {
    let ptr = symbol_addr!(_smanifest).cast();
    unsafe { &*ptr }
}

pub fn inactive() -> &'static Manifest {
    let ptr = (active() as *const Manifest).map_addr(crate::flash::flip_bank);
    unsafe { &*ptr }
}

#[repr(C)]
pub struct Manifest {
    pub signature: Signature,
    pub usage_constraints: UsageConstraints,
    pub public_key: PublicKey,
    pub address_translation: [u8; 4],
    pub identifier: [u8; 4],
    pub manifest_version: [u8; 4],
    pub signed_region_end: [u8; 4],
    pub length: [u8; 4],
    pub version_major: [u8; 4],
    pub version_minor: [u8; 4],
    pub security_version: [u8; 4],
    pub timestamp: [u8; 8],
    pub binding_value: [u8; 32],
    pub max_key_version: [u8; 4],
    pub code_start: [u8; 4],
    pub code_end: [u8; 4],
    pub entry_point: [u8; 4],
    pub extensions: [Extension; 15],
}

#[repr(C)]
pub union Signature {
    pub rsa: [u8; 384],
    pub ecdsa: [u8; 64],
}

#[repr(C)]
pub struct UsageConstraints {
    pub selector_bits: [u8; 4],
    pub device_id: [u8; 32],
    pub manuf_state_creator: [u8; 4],
    pub manuf_state_owner: [u8; 4],
    pub life_cycle_state: [u8; 4],
}

#[repr(C)]
pub union PublicKey {
    pub rsa: [u8; 384],
    pub ecdsa: [u8; 64],
}

#[repr(C)]
pub struct Extension {
    pub identifier: [u8; 4],
    pub offset: [u8; 4],
}

const _: () = assert!(offset_of!(Manifest, signature) == 0);
const _: () = assert!(offset_of!(Manifest, usage_constraints) == 384);
const _: () = assert!(offset_of!(Manifest, public_key) == 432);
const _: () = assert!(offset_of!(Manifest, address_translation) == 816);
const _: () = assert!(offset_of!(Manifest, identifier) == 820);
const _: () = assert!(offset_of!(Manifest, manifest_version) == 824);
const _: () = assert!(offset_of!(Manifest, signed_region_end) == 828);
const _: () = assert!(offset_of!(Manifest, length) == 832);
const _: () = assert!(offset_of!(Manifest, version_major) == 836);
const _: () = assert!(offset_of!(Manifest, version_minor) == 840);
const _: () = assert!(offset_of!(Manifest, security_version) == 844);
const _: () = assert!(offset_of!(Manifest, timestamp) == 848);
const _: () = assert!(offset_of!(Manifest, binding_value) == 856);
const _: () = assert!(offset_of!(Manifest, max_key_version) == 888);
const _: () = assert!(offset_of!(Manifest, code_start) == 892);
const _: () = assert!(offset_of!(Manifest, code_end) == 896);
const _: () = assert!(offset_of!(Manifest, entry_point) == 900);
const _: () = assert!(offset_of!(Manifest, extensions) == 904);
const _: () = assert!(size_of::<Manifest>() == 1024);
