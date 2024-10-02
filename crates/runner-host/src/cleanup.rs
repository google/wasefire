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

use std::io::Write;
use std::sync::Mutex;

pub type Cleanup = Box<dyn FnOnce() + Send>;

#[cfg(feature = "unix")]
pub fn push(cleanup: Cleanup) {
    CLEANUP.lock().unwrap().push(cleanup);
}

pub fn shutdown(status: i32) -> ! {
    let cleanups = std::mem::take(&mut *CLEANUP.lock().unwrap());
    for cleanup in cleanups {
        cleanup();
    }
    wasefire_logger::flush();
    _ = std::io::stdout().flush();
    _ = std::io::stderr().flush();
    std::process::exit(status)
}

static CLEANUP: Mutex<Vec<Cleanup>> = Mutex::new(Vec::new());
