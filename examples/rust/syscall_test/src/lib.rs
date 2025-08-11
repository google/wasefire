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

use alloc::vec;
wasefire::applet!();

fn main() {
    test_error();
    test_read();
    test_write();
    test_alloc();
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
