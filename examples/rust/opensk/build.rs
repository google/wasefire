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

extern crate alloc;

use std::path::PathBuf;

use uuid::Uuid;

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=crypto_data/aaguid.txt");
    let content = std::fs::read_to_string("crypto_data/aaguid.txt").unwrap();
    let aaguid = Uuid::parse_str(content.trim()).unwrap();
    std::fs::write(out_dir.join("opensk_aaguid.bin"), aaguid.as_bytes()).unwrap();
}
