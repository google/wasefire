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

#![no_std]
wasefire::applet!();


use core::sync::atomic::*;

static atomic_u32:AtomicU32 = AtomicU32::new(0);
static atomic_i32:AtomicI32 = AtomicI32::new(0);
static atomic_bool:AtomicBool = AtomicBool::new(false);

fn main() {
    debug!("hello world");
    fence(Ordering::Acquire);
}

#[no_mangle]
pub fn set() {
    atomic_i32.store(8888888, Ordering::SeqCst);
    atomic_u32.store(8888888, Ordering::SeqCst);
    atomic_i32.fetch_sub(1, Ordering::SeqCst);
    
    atomic_bool.load(Ordering::SeqCst);
    atomic_bool.store(false, Ordering::SeqCst);
    let r = atomic_bool.fetch_or(true, Ordering::SeqCst);
    let r2 = atomic_bool.fetch_and(r, Ordering::SeqCst);
    let r3 = atomic_bool.fetch_xor(r2, Ordering::SeqCst);
}

#[no_mangle]
pub fn get() -> i32 {
    atomic_i32.fetch_add(10, Ordering::SeqCst);
    atomic_i32.load(Ordering::SeqCst)
}