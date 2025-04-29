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

use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static ALLOCATOR: Heap = Heap::empty();

macro_rules! addr_of_sym {
    ($sym:ident) => {{
        unsafe extern "C" {
            static mut $sym: u32;
        }
        core::ptr::addr_of_mut!($sym) as usize
    }};
}

pub(crate) fn init() {
    #[cfg(feature = "target-nordic")]
    let (sheap, eheap) = (cortex_m_rt::heap_start() as usize, addr_of_sym!(_stack_end));
    #[cfg(feature = "target-riscv")]
    let (sheap, eheap) = (addr_of_sym!(_sheap), addr_of_sym!(_eheap));
    assert!(sheap < eheap);
    #[cfg(feature = "target-nordic")]
    {
        let (sram, eram) = (addr_of_sym!(_ram_start), addr_of_sym!(_ram_end));
        crate::println!(
            "Size: data={} heap={} stack={}",
            sheap - sram,
            eheap - sheap,
            eram - eheap
        );
    }
    // SAFETY: Called only once before any allocation.
    unsafe { ALLOCATOR.init(sheap, eheap - sheap) };
}

#[cfg(feature = "target-nordic")]
pub(crate) mod usage {
    use core::sync::atomic::{AtomicU32, Ordering};

    pub(crate) fn print() {
        static HEAP: Peak = Peak::new();
        static STACK: Peak = Peak::new();
        crate::println!("Peak usage: heap={} stack={}", HEAP.update(heap()), STACK.update(stack()));
    }

    struct Peak(AtomicU32);

    impl Peak {
        const fn new() -> Self {
            Peak(AtomicU32::new(0))
        }

        fn update(&self, x: u32) -> u32 {
            core::cmp::max(self.0.fetch_max(x, Ordering::Relaxed), x)
        }
    }

    fn heap() -> u32 {
        super::ALLOCATOR.used() as u32
    }

    fn stack() -> u32 {
        let x = addr_of_sym!(_stack_start) as u32;
        x - &x as *const _ as u32
    }
}
