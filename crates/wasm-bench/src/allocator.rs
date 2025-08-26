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

use core::alloc::{GlobalAlloc, Layout};
use core::ops::Range;
use core::sync::atomic::Ordering::Relaxed;

use embedded_alloc::LlffHeap;
use wasefire_sync::AtomicUsize;

macro_rules! addr_of_sym {
    ($sym:ident) => {{
        unsafe extern "C" {
            static mut $sym: u32;
        }
        (&raw mut $sym).addr()
    }};
}

pub(crate) fn init() {
    let Regions { data, heap, stack } = regions();
    crate::println!("Size: data={} heap={} stack={}", data.len(), heap.len(), stack.len());
    // SAFETY: Called only once before any allocation.
    unsafe { ALLOCATOR.0.init(heap.start, heap.len()) };
    for ptr in stack.start .. cur_stack(0) {
        // SAFETY: That part of the stack should be unused (write_volatile should be inlined).
        unsafe { (ptr as *mut u8).write_volatile(0) };
    }
}

pub(crate) fn done() {
    let heap = PEAK.load(Relaxed);
    let stack = peak_stack(regions().stack);
    crate::println!("Peak usage: heap={heap} stack={stack}");
}

static PEAK: AtomicUsize = AtomicUsize::new(0);

#[inline(never)]
fn cur_stack(x: usize) -> usize {
    x + &x as *const _ as usize
}

fn peak_stack(region: Range<usize>) -> usize {
    for ptr in region.clone() {
        // SAFETY: That part of the stack should be unused (read_volatile should be inlined).
        if unsafe { (ptr as *const u8).read_volatile() } != 0 {
            return region.end - ptr;
        }
    }
    0
}

struct Regions {
    data: Range<usize>,
    heap: Range<usize>,
    stack: Range<usize>,
}

#[cfg(feature = "target-nordic")]
fn regions() -> Regions {
    let (sheap, eheap) = (cortex_m_rt::heap_start() as usize, addr_of_sym!(_stack_end));
    let (sram, eram) = (addr_of_sym!(_ram_start), addr_of_sym!(_ram_end));
    assert!(sram <= sheap && sheap < eheap && eheap < eram);
    Regions { data: sram .. sheap, heap: sheap .. eheap, stack: eheap .. eram }
}

#[cfg(feature = "target-riscv")]
fn regions() -> Regions {
    let sram = addr_of_sym!(sram);
    let sheap = riscv_rt::heap_start() as usize;
    let eheap = sheap + addr_of_sym!(_heap_size);
    let sstack = addr_of_sym!(sstack);
    let estack = addr_of_sym!(_stack_start);
    assert!(sram <= sheap && sstack < estack);
    Regions { data: sram .. sheap, heap: sheap .. eheap, stack: sstack .. estack }
}

#[global_allocator]
static ALLOCATOR: Heap = Heap(LlffHeap::empty());

struct Heap(LlffHeap);

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.0.alloc(layout) };
        PEAK.fetch_max(self.0.used(), Relaxed);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.0.dealloc(ptr, layout) }
    }
}
