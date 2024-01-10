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

//! Tests that the sync module is working properly.

#![no_std]
wasefire::applet!();

use alloc::rc::Rc;
use core::cell::Cell;

fn main() {
    test_mutex();
    test_atomic();
    debug::exit(true);
}

fn test_mutex() {
    debug!("test_mutex(): Mutex operations should work.");
    static GLOBAL: sync::Mutex<i32> = sync::Mutex::new(0);
    debug!("- read initial value");
    debug::assert_eq(&*GLOBAL.lock(), &0);
    debug!("- write new value");
    *GLOBAL.lock() = 1;
    debug!("- read updated value");
    debug::assert_eq(&*GLOBAL.lock(), &1);
    let _lock = GLOBAL.lock();
    debug!("- try to double lock from the same thread");
    debug::assert(GLOBAL.try_lock().is_none());
    debug!("- try to double lock from a callback");
    let has_run = Rc::new(Cell::new(false));
    let timer = timer::Timer::new({
        let has_run = has_run.clone();
        move || {
            has_run.set(true);
            debug::assert(GLOBAL.try_lock().is_none());
        }
    });
    timer.start_ms(timer::Oneshot, 100);
    scheduling::wait_for_callback();
    debug::assert(has_run.get());
}

fn test_atomic() {
    debug!("test_atomic(): Atomic operations should work.");
    static GLOBAL: sync::AtomicI32 = sync::AtomicI32::new(0);
    debug!("- read initial value");
    debug::assert_eq(&GLOBAL.load(sync::Ordering::SeqCst), &0);
    debug!("- write new value");
    GLOBAL.store(1, sync::Ordering::SeqCst);
    debug!("- read updated value");
    debug::assert_eq(&GLOBAL.load(sync::Ordering::SeqCst), &1);
}
