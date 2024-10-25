// Copyright 2023 Google LLC
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

//! This crate provides stubs for parts of the applet API.
//!
//! By default, when using the applet API with the `native` feature, all calls to the applet API
//! will panic. This crate provides a working implementation for some parts of the API (like the
//! crypto module).
//!
//! Note that this crate doesn't expose anything. To use it, you just need to link it:
//!
//! ```
//! use wasefire_stub as _;
//! ```

#![feature(try_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

use wasefire_error::Error;

mod crypto;
mod debug;
mod rng;
mod scheduling;

fn convert(x: Result<u32, Error>) -> isize {
    Error::encode(x) as isize
}

fn convert_bool(x: Result<bool, Error>) -> isize {
    convert(x.map(|x| x as u32))
}

fn convert_unit(x: Result<(), Error>) -> isize {
    convert(x.map(|()| 0))
}
