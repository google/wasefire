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
wasefire::applet!();

use alloc::rc::Rc;
use core::cell::Cell;

#[cfg(feature = "human")]
const SECOND_MS: usize = 1000;
#[cfg(not(feature = "human"))]
const SECOND_MS: usize = 100;

fn main() {
    #[cfg(not(feature = "human"))]
    debug!("Running at 10x speed (use --features=human to run at normal speed).");
    test_oneshot();
    test_periodic(true);
    test_periodic(false);
    test_oneshot_cancel();
    test_periodic_cancel();
    test_cancel_callback();
    test_empty();
    debug::exit(true);
}

fn test_periodic(explicit_stop: bool) {
    debug!("test_periodic({}): This should count seconds from 1 to 5.", explicit_stop);
    let i = Rc::new(Cell::new(0));
    let timer = timer::Timer::new({
        let i = i.clone();
        move || {
            let j = i.get();
            debug!("- {}", j + 1);
            i.set(j + 1);
        }
    });
    debug!("+ start timer");
    timer.start_ms(timer::Periodic, SECOND_MS);
    while i.get() < 5 {
        scheduling::wait_for_callback();
    }
    if explicit_stop {
        debug!("+ stop timer");
        timer.stop();
    }
    debug::assert_eq(&i.get(), &5);
}

fn test_oneshot() {
    debug!("test_oneshot(): This should print a message after 1 second.");
    let done = Rc::new(Cell::new(false));
    let timer = timer::Timer::new({
        let done = done.clone();
        move || {
            debug!("- done");
            done.set(true);
        }
    });
    debug!("+ start timer");
    timer.start_ms(timer::Oneshot, SECOND_MS);
    scheduling::wait_until(|| done.get());
    debug::assert(done.get());
}

fn test_oneshot_cancel() {
    debug!("test_oneshot_cancel(): This should cancel a 5 seconds timer after 1 second.");
    let done = Rc::new(Cell::new(false));
    let timer = timer::Timer::new({
        let done = done.clone();
        move || done.set(true)
    });
    debug!("+ start timer");
    timer.start_ms(timer::Oneshot, 5 * SECOND_MS);
    debug!("+ sleep");
    timer::sleep_ms(SECOND_MS);
    debug!("+ stop timer");
    timer.stop();
    debug::assert(!done.get());
}

fn test_periodic_cancel() {
    debug!("test_periodic_cancel(): This should cancel a periodic timer after 3.5 seconds.");
    let i = Rc::new(Cell::new(0));
    let timer = timer::Timer::new({
        let i = i.clone();
        move || {
            let j = i.get();
            debug!("- {}", j + 1);
            i.set(j + 1);
        }
    });
    debug!("+ start timer");
    timer.start_ms(timer::Periodic, SECOND_MS);
    timer::sleep_ms(7 * SECOND_MS / 2);
    debug!("+ stop timer");
    timer.stop();
    debug::assert_eq(&i.get(), &3);
}

fn test_cancel_callback() {
    debug!("test_cancel_callback(): This should cancel a pending callback.");
    let has_run = Rc::new(Cell::new(false));
    let first = timer::Timer::new({
        let has_run = has_run.clone();
        move || has_run.set(true)
    });
    first.start_ms(timer::Oneshot, SECOND_MS);
    while scheduling::num_pending_callbacks() == 0 {}
    debug!("- callback scheduled");
    drop(first);
    debug::assert_eq(&scheduling::num_pending_callbacks(), &0);
    debug::assert(!has_run.get());
}

fn test_empty() {
    debug!("test_empty(): This should instantly print a message.");
    let done = Rc::new(Cell::new(false));
    let timer = timer::Timer::new({
        let done = done.clone();
        move || done.set(true)
    });
    debug!("+ start timer");
    timer.start_ms(timer::Oneshot, 0);
    scheduling::wait_until(|| done.get());
    debug!("- done");
    debug::assert(done.get());
}
