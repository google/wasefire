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

//! Tests that calling unknown host functions links and returns an error.

#![no_std]
wasefire::applet!();

#[cfg(feature = "native")]
pub use native::*;
use wasefire_error::Code;

fn main() {
    debug!("Calling unknown host functions:");
    test(|| unsafe { test_only_0() });
    test(|| unsafe { test_only_1(0) });
    test(|| unsafe { test_only_1b(0) });
    test(|| unsafe { test_only_10(0, 0, 0, 0, 0, 0, 0, 0, 0, 0) });
    scheduling::exit();
}

fn test(f: impl FnOnce() -> isize) {
    let actual = f();
    debug!("- {:08x} {:?}", actual as usize, Error::decode(actual as i32));
    let expected = Error::encode(Err(Error::world(Code::NotImplemented))) as isize;
    assert_eq!(actual, expected);
}

#[cfg(not(feature = "native"))]
extern "C" {
    fn test_only_0() -> isize;
    fn test_only_1(_: usize) -> isize;
    fn test_only_1b(_: usize) -> isize;
    fn test_only_10(
        _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize,
        _: usize,
    ) -> isize;
}

#[cfg(feature = "native")]
mod native {
    use core::ffi::{c_char, CStr};

    extern "C" {
        fn env_dispatch(link: *const c_char, params: *const u32) -> isize;
    }

    fn test_only(link: &str, params: &[u32]) -> isize {
        let link = CStr::from_bytes_with_nul(link.as_bytes()).unwrap();
        unsafe { env_dispatch(link.as_ptr(), params.as_ptr()) }
    }

    pub unsafe fn test_only_0() -> isize {
        test_only("test_only_0\0", &[])
    }

    pub unsafe fn test_only_1(x: usize) -> isize {
        test_only("test_only_1\0", &[x as u32])
    }

    pub unsafe fn test_only_1b(x: usize) -> isize {
        test_only("test_only_1b\0", &[x as u32])
    }

    pub unsafe fn test_only_10(
        x0: usize, x1: usize, x2: usize, x3: usize, x4: usize, x5: usize, x6: usize, x7: usize,
        x8: usize, x9: usize,
    ) -> isize {
        test_only(
            "test_only_10\0",
            &[
                x0 as u32, x1 as u32, x2 as u32, x3 as u32, x4 as u32, x5 as u32, x6 as u32,
                x7 as u32, x8 as u32, x9 as u32,
            ],
        )
    }
}
