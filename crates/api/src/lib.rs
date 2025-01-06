// Copyright 2022 Google LLC
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

#![cfg_attr(feature = "host", doc = "Platform-side of the applet API.")]
#![cfg_attr(feature = "wasm", doc = include_str!("wasm.md"))]
#![cfg_attr(feature = "wasm", doc = "\n## Language-agnostic documentation\n")]
#![cfg_attr(feature = "wasm", doc = include_str!("api.md"))]
#![no_std]
#![cfg_attr(all(feature = "wasm", feature = "native"), feature(linkage))]
#![cfg_attr(feature = "host", feature(never_type))]
#![feature(doc_auto_cfg)]

extern crate alloc;

#[cfg(feature = "host")]
pub use host::*;
use wasefire_error as _;
use wasefire_one_of::exactly_one_of;

#[cfg(feature = "host")]
mod host;
#[cfg(feature = "wasm")]
pub(crate) mod wasm;

exactly_one_of!["host", "wasm"];

#[cfg(feature = "wasm")]
wasefire_applet_api_macro::wasm!();

#[cfg(feature = "host")]
wasefire_applet_api_macro::host!();
