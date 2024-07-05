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

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let memory = if std::env::var_os("CARGO_FEATURE_TARGET_NORDIC").is_some() {
        Some(include_bytes!("memory-nordic.x").as_slice())
    } else if std::env::var_os("CARGO_FEATURE_TARGET_RISCV").is_some() {
        Some(include_bytes!("memory-riscv.x").as_slice())
    } else {
        None
    };
    if let Some(memory) = memory {
        let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
        println!("cargo:rustc-link-search={}", out.display());
        File::create(out.join("memory.x")).unwrap().write_all(memory).unwrap();
    }
}
