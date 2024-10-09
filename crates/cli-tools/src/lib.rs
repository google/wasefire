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

//! Provides command-line utilities for Wasefire CLI.
//!
//! This library is also used for the internal maintenance CLI of Wasefire called xtask.

#![feature(async_fn_track_caller)]
#![feature(doc_auto_cfg)]
#![feature(never_type)]
#![feature(path_add_extension)]
#![feature(try_find)]

macro_rules! debug {
    ($($x:tt)*) => {
        if log::log_enabled!(log::Level::Warn) {
            print!("\x1b[1;36m");
            print!($($x)*);
            println!("\x1b[m");
        }
    };
}

#[cfg(feature = "action")]
pub mod action;
pub mod changelog;
pub mod cmd;
pub mod error;
pub mod fs;
