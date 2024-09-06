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

use core::ptr::addr_of_mut;

use embedded_alloc::Heap;

#[global_allocator]
static ALLOCATOR: Heap = Heap::empty();

pub fn init() {
    extern "C" {
        static mut __sheap: u32;
        static mut __eheap: u32;
    }
    let sheap = addr_of_mut!(__sheap) as usize;
    let eheap = addr_of_mut!(__eheap) as usize;
    assert!(sheap < eheap);
    // Unsafe: Called only once before any allocation.
    unsafe { ALLOCATOR.init(sheap, eheap - sheap) }
}
