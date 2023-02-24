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

//! Tests that timers are working properly.

#![no_std]

extern crate alloc;

use alloc::rc::Rc;
use core::cell::Cell;

use wasefire::*;

#[no_mangle]
pub extern "C" fn main() {
    test_oneshot();
    test_periodic(true);
    test_periodic(false);
    test_oneshot_cancel();
    test_periodic_cancel();
    test_cancel_callback();
    println!("End of tests.");
}

fn test_periodic(explicit_stop: bool) {
    println!("test_periodic({}): This should count seconds from 1 to 5.", explicit_stop);
    let i = Rc::new(Cell::new(0));
    let timer = clock::Timer::new({
        let i = i.clone();
        move || {
            let j = i.get();
            println!("- {}", j + 1);
            i.set(j + 1);
        }
    });
    println!("+ start timer");
    timer.start_ms(clock::Periodic, 1000);
    while i.get() < 5 {
        scheduling::wait_for_callback();
    }
    if explicit_stop {
        println!("+ stop timer");
        timer.stop();
    }
    assert_eq!(i.get(), 5);
}

fn test_oneshot() {
    println!("test_oneshot(): This should print a message after 1 second.");
    let done = Rc::new(Cell::new(false));
    let timer = clock::Timer::new({
        let done = done.clone();
        move || {
            println!("- done");
            done.set(true);
        }
    });
    println!("+ start timer");
    timer.start_ms(clock::Oneshot, 1000);
    scheduling::wait_until(|| done.get());
    assert!(done.get());
}

fn test_oneshot_cancel() {
    println!("test_oneshot_cancel(): This should cancel a 5 seconds timer after 1 second.");
    let done = Rc::new(Cell::new(false));
    let timer = clock::Timer::new({
        let done = done.clone();
        move || done.set(true)
    });
    println!("+ start timer");
    timer.start_ms(clock::Oneshot, 5000);
    println!("+ sleep");
    clock::sleep_ms(1000);
    println!("+ stop timer");
    timer.stop();
    assert!(!done.get());
}

fn test_periodic_cancel() {
    println!("test_periodic_cancel(): This should cancel a periodic timer after 3.5 seconds.");
    let i = Rc::new(Cell::new(0));
    let timer = clock::Timer::new({
        let i = i.clone();
        move || {
            let j = i.get();
            println!("- {}", j + 1);
            i.set(j + 1);
        }
    });
    println!("+ start timer");
    timer.start_ms(clock::Periodic, 1000);
    clock::sleep_ms(3500);
    println!("+ stop timer");
    timer.stop();
    assert_eq!(i.get(), 3);
}

fn test_cancel_callback() {
    println!("test_cancel_callback(): This should cancel a pending callback.");
    let has_run = Rc::new(Cell::new(false));
    let first = clock::Timer::new({
        let has_run = has_run.clone();
        move || has_run.set(true)
    });
    first.start_ms(clock::Oneshot, 1000);
    while scheduling::num_pending_callbacks() == 0 {}
    println!("- callback scheduled");
    drop(first);
    assert_eq!(scheduling::num_pending_callbacks(), 0);
    assert!(!has_run.get());
}
