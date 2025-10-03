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

#[cfg(not(feature = "test-vendor"))]
use embedded_alloc::TlsfHeap as Heap;
use wasefire_common::addr_of_symbol;
#[cfg(feature = "test-vendor")]
pub use wrapper::*;

#[global_allocator]
static ALLOCATOR: Heap = Heap::empty();

pub fn init() {
    let sheap = addr_of_symbol!(__sheap);
    let eheap = addr_of_symbol!(__eheap);
    assert!(sheap < eheap);
    #[cfg(feature = "test-vendor")]
    wrapper::USAGE.store(sheap - addr_of_symbol!(_rheap), core::sync::atomic::Ordering::Relaxed);
    // SAFETY: Called only once before any allocation.
    unsafe { ALLOCATOR.init(sheap, eheap - sheap) }
}

#[cfg(feature = "test-vendor")]
mod wrapper {
    use core::alloc::Layout;
    use core::sync::atomic::Ordering::Relaxed;

    use embedded_alloc::TlsfHeap;
    use wasefire_sync::AtomicUsize;

    pub static USAGE: AtomicUsize = AtomicUsize::new(0);
    static PEAK: AtomicUsize = AtomicUsize::new(0);

    pub struct Heap(TlsfHeap);

    unsafe impl core::alloc::GlobalAlloc for Heap {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let result = unsafe { self.0.alloc(layout) };
            if !result.is_null() {
                let usage = USAGE.fetch_add(layout.size(), Relaxed) + layout.size();
                PEAK.fetch_max(usage, Relaxed);
            }
            result
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            USAGE.fetch_sub(layout.size(), Relaxed);
            unsafe { self.0.dealloc(ptr, layout) }
        }
    }

    impl Heap {
        pub const fn empty() -> Self {
            Heap(TlsfHeap::empty())
        }

        pub unsafe fn init(&self, start: usize, size: usize) {
            unsafe { self.0.init(start, size) }
        }
    }

    pub fn peak() -> usize {
        PEAK.swap(0, Relaxed)
    }
}
