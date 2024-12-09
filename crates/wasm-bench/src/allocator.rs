// Copyright 2024 Google LLC
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

use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static ALLOCATOR: Heap = Heap::empty();

macro_rules! addr_of_sym {
    ($sym:ident) => {{
        unsafe extern "C" {
            static mut $sym: u32;
        }
        addr_of_mut!($sym) as usize
    }};
}

pub(crate) fn init() {
    #[cfg(feature = "target-nordic")]
    let (sheap, eheap) = (addr_of_sym!(__sheap), addr_of_sym!(__eheap));
    #[cfg(feature = "target-riscv")]
    let (sheap, eheap) = (addr_of_sym!(_sheap), addr_of_sym!(_eheap));
    assert!(sheap < eheap);
    // SAFETY: Called only once before any allocation.
    unsafe { ALLOCATOR.init(sheap, eheap - sheap) };
}

#[cfg(feature = "target-nordic")]
pub(crate) fn usage() -> usize {
    ALLOCATOR.used()
}
