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

#![no_std]
wasefire::applet!();

use core::sync::atomic::*;

static ATOMIC_U32: AtomicU32 = AtomicU32::new(0);
static ATOMIC_I32: AtomicI32 = AtomicI32::new(0);
static ATOMIC_BOOL: AtomicBool = AtomicBool::new(false);

fn main() {
    debug!("hello world");
    fence(Ordering::Acquire);
}

#[no_mangle]
pub fn set() {
    ATOMIC_I32.store(8888888, Ordering::SeqCst);
    ATOMIC_U32.store(8888888, Ordering::SeqCst);
    ATOMIC_I32.fetch_sub(1, Ordering::SeqCst);

    ATOMIC_BOOL.load(Ordering::SeqCst);
    ATOMIC_BOOL.store(false, Ordering::SeqCst);
    let r = ATOMIC_BOOL.fetch_or(true, Ordering::SeqCst);
    let r2 = ATOMIC_BOOL.fetch_and(r, Ordering::SeqCst);
    ATOMIC_BOOL.fetch_xor(r2, Ordering::SeqCst);

}

#[no_mangle]
pub fn get() -> i32 {
    ATOMIC_I32.fetch_add(10, Ordering::SeqCst);
    ATOMIC_I32.load(Ordering::SeqCst)
}
