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

//! Wasefire USB protocol.
//!
//! This crate implements a USB RPC protocol that supports the Wasefire platform protocol.

#![no_std]
#![feature(never_type)]

#[cfg(feature = "device")]
const _DEVICE_OR_HOST: () = ();
#[cfg(feature = "host")]
const _DEVICE_OR_HOST: () = ();

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "device")]
pub use device::*;
#[cfg(feature = "host")]
pub use host::*;

pub mod common;
#[cfg(feature = "device")]
mod device;
#[cfg(feature = "host")]
mod host;
