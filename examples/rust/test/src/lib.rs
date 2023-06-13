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

#![no_std]
wasefire::applet!();

use alloc::boxed::Box;
use core::cell::Cell;

fn main() {
    let byte = make();
    usb::serial::write_all(&[byte.get()]).unwrap();
}

fn make() -> Box<Cell<u8>> {
    Box::new(Cell::new(18))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make() {
        // We want to make sure types in core and alloc also work in tests.
        assert_eq!(make().get(), 18);
    }

    #[test]
    fn test_rng() {
        use wasefire_applet_api::rng::fill_bytes::{Params, Results};
        #[no_mangle]
        unsafe extern "C" fn rb(params: Params) -> Results {
            let Params { ptr, len } = params;
            core::slice::from_raw_parts_mut(ptr, len).fill(42);
            Results { res: 0 }
        }
        let mut buf = [0];
        rng::fill_bytes(&mut buf).unwrap();
        assert_eq!(buf, [42]);
    }
}
