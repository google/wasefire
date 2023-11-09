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

use alloc::boxed::Box;
use core::cell::RefCell;
use core::ffi::{c_char, CStr};

use critical_section::Mutex;
use portable_atomic::AtomicBool;
use portable_atomic::Ordering::SeqCst;
use wasefire_board_api::Api as Board;

#[cfg(feature = "debug")]
use crate::perf::Slot;
use crate::Scheduler;

pub(crate) trait ErasedScheduler: Send {
    fn dispatch(&mut self, link: &CStr, params: *const u32, results: *mut u32);
    fn flush_events(&mut self);
    fn process_event(&mut self);
    #[cfg(feature = "debug")]
    fn perf_record(&mut self, slot: Slot);
}

impl<B: Board> ErasedScheduler for Scheduler<B> {
    fn dispatch(&mut self, link: &CStr, params: *const u32, results: *mut u32) {
        self.dispatch(link, params, results);
    }

    fn flush_events(&mut self) {
        self.flush_events();
    }

    fn process_event(&mut self) {
        self.process_event();
    }

    #[cfg(feature = "debug")]
    fn perf_record(&mut self, slot: Slot) {
        self.perf.record(slot);
    }
}

// TODO: There should be a Mutex that doesn't disable interrupts for the whole critical section. We
// implement it manually for now.
static BORROWED: AtomicBool = AtomicBool::new(false);
static mut SCHEDULER: Option<Box<dyn ErasedScheduler>> = None;
#[allow(clippy::type_complexity)]
static CALLBACK: Mutex<RefCell<Option<Box<dyn Fn() + Send>>>> = Mutex::new(RefCell::new(None));

fn lock<R>(operation: impl FnOnce() -> R) -> R {
    assert!(!BORROWED.swap(true, SeqCst));
    let result = operation();
    assert!(BORROWED.swap(false, SeqCst));
    result
}

pub(crate) fn set_scheduler<B: Board>(scheduler: Scheduler<B>) {
    lock(|| {
        assert!(unsafe { SCHEDULER.is_none() });
        unsafe { SCHEDULER = Some(Box::new(scheduler)) };
    });
}

pub(crate) fn with_scheduler<R>(f: impl FnOnce(&mut dyn ErasedScheduler) -> R) -> R {
    lock(|| f(unsafe { SCHEDULER.as_deref_mut().unwrap() }))
}

pub(crate) fn schedule_callback(callback: Box<dyn Fn() + Send>) {
    assert!(critical_section::with(|cs| CALLBACK.replace(cs, Some(callback))).is_none());
}

pub(crate) fn execute_callback() {
    if let Some(callback) = critical_section::with(|cs| CALLBACK.take(cs)) {
        #[cfg(feature = "debug")]
        with_scheduler(|x| x.perf_record(Slot::Platform));
        callback();
        #[cfg(feature = "debug")]
        with_scheduler(|x| x.perf_record(Slot::Applets));
    }
}

#[no_mangle]
extern "C" fn env_dispatch(link: *const c_char, params: *const u32, results: *mut u32) {
    let link = unsafe { CStr::from_ptr(link) };
    with_scheduler(|scheduler| {
        #[cfg(feature = "debug")]
        scheduler.perf_record(Slot::Applets);
        scheduler.flush_events();
        scheduler.dispatch(link, params, results);
    });
    execute_callback();
    #[cfg(feature = "debug")]
    with_scheduler(|x| x.perf_record(Slot::Platform));
}
