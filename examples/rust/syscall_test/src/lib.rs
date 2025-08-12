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

//! Tests that the board-specific syscall API is working properly.

#![no_std]
wasefire::applet!();

use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;

use wasefire::vendor::syscall;

fn main() {
    test_error();
    test_read();
    test_write();
    test_alloc();
    test_handler();
    scheduling::exit();
}

fn test_error() {
    debug!("test_error(): Check that errors are correctly converted.");
    for (x, r) in [
        (0, Ok(0)),
        (0x7fffffff, Ok(0x7fffffff)),
        (0xff000000, Err(Error::new(0xff, 0xffff))),
        (0xfffefffd, Err(Error::user(2))),
        (0xffffffff, Err(Error::default())),
    ] {
        debug!("- {x:08x} -> {r:?}");
        assert_eq!(unsafe { syscall(0, 0, 0, x) }, r);
    }
}

fn test_read() {
    debug!("test_read(): Check that platform can read memory.");
    for len in [0, 1, 7, 8] {
        let data = rng::bytes(len).unwrap();
        debug!("- {data:02x?}");
        let expected = data.iter().map(|&x| x as usize).sum();
        let actual = unsafe { syscall(0, 1, len, data.as_ptr().addr()) }.unwrap();
        assert_eq!(actual, expected);
    }
}

fn test_write() {
    debug!("test_write(): Check that platform can write memory.");
    for len in [0, 1, 7, 8] {
        let mut data = vec![0u8; len];
        assert_eq!(unsafe { syscall(0, 2, len, data.as_mut_ptr().addr()) }.unwrap(), len);
        debug!("- {data:02x?}");
        assert!(data.iter().zip((0 .. len).rev()).all(|(&x, y)| x == y as u8));
    }
}

fn test_alloc() {
    debug!("test_alloc(): Check that platform can allocated memory.");
    for len in [1, 7, 8, 9] {
        let ptr = unsafe { syscall(0, 3, len, 0) }.unwrap();
        let data = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
        debug!("- {data:02x?}");
        assert!(data.iter().all(|&x| x == len as u8));
    }
}

fn test_handler() {
    debug!("test_handler(): Check that platform can register handler.");
    type Call = RefCell<Vec<Option<u32>>>;
    let calls: Call = RefCell::new(Vec::new());
    let data = &calls as *const _ as usize;
    extern "C" fn foo(data: *const u8) {
        let calls = unsafe { &*data.cast::<Call>() };
        calls.borrow_mut().push(None);
    }
    extern "C" fn bar(data: *const u8, value: u32) {
        let calls = unsafe { &*data.cast::<Call>() };
        calls.borrow_mut().push(Some(value));
    }
    debug!("- register handlers for Foo and Bar events");
    unsafe { syscall(0, 4, foo as usize, data) }.unwrap();
    unsafe { syscall(0, 5, bar as usize, data) }.unwrap();
    debug!("- trigger a Foo event and make sure the Foo handler is called");
    unsafe { syscall(0, 7, 0, 0) }.unwrap();
    assert_eq!(scheduling::num_pending_callbacks(), 1);
    assert_eq!(*calls.borrow(), []);
    scheduling::wait_for_callback();
    assert_eq!(scheduling::num_pending_callbacks(), 0);
    assert_eq!(*calls.borrow(), [None]);
    debug!("- trigger another Foo event and make sure the foo handler is not called");
    // The Foo handler was automatically unregistered by the previous Foo event. The scheduler
    // discards events when it has no registered handler.
    unsafe { syscall(0, 7, 0, 0) }.unwrap();
    assert_eq!(scheduling::num_pending_callbacks(), 0);
    debug!("- trigger multiple Bar events with values 42, 13, and 13");
    // The scheduler will merge equal events.
    unsafe { syscall(0, 7, 1, 42) }.unwrap();
    unsafe { syscall(0, 7, 1, 13) }.unwrap();
    unsafe { syscall(0, 7, 1, 13) }.unwrap();
    debug!("- make sure the bar handler is called only twice with 42 and 13");
    assert_eq!(scheduling::num_pending_callbacks(), 2);
    assert_eq!(*calls.borrow(), [None]);
    scheduling::wait_for_callback();
    assert_eq!(scheduling::num_pending_callbacks(), 1);
    assert_eq!(*calls.borrow(), [None, Some(42)]);
    scheduling::wait_for_callback();
    assert_eq!(scheduling::num_pending_callbacks(), 0);
    assert_eq!(*calls.borrow(), [None, Some(42), Some(13)]);
    debug!("- unregister the handler for Bar events");
    unsafe { syscall(0, 6, 1, 0) }.unwrap();
}
