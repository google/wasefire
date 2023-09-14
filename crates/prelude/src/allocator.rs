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

use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr::{null_mut, NonNull};

use const_default::ConstDefault;
use rlsf::Tlsf;

use crate::debug::println;

#[no_mangle]
pub extern "C" fn init() {
    // TODO: Make sure it's called once.
    const SIZE: usize = 32768;
    static mut POOL: MaybeUninit<[u8; SIZE]> = MaybeUninit::uninit();
    let pool_ptr = unsafe { NonNull::new(POOL.as_mut_ptr()).unwrap() };
    let size = unsafe { ALLOCATOR.0.borrow_mut().insert_free_block_ptr(pool_ptr) };
    assert!(size > NonZeroUsize::new(SIZE / 2));
}

#[no_mangle]
pub extern "C" fn alloc(size: i32, align: i32) -> i32 {
    let layout = match Layout::from_size_align(size as usize, align as usize) {
        Ok(x) => x,
        Err(_) => return 0,
    };
    unsafe { ALLOCATOR.alloc(layout) as i32 }
}

struct Allocator(RefCell<Tlsf<'static, u8, u8, 8, 8>>);

#[global_allocator]
static mut ALLOCATOR: Allocator = Allocator(RefCell::new(Tlsf::DEFAULT));

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.0.borrow_mut().allocate(layout) {
            None => null_mut(),
            Some(x) => x.as_ptr(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.borrow_mut().deallocate(NonNull::new(ptr).unwrap(), layout.align())
    }
}

#[alloc_error_handler]
fn handle_oom(_: core::alloc::Layout) -> ! {
    println("OOM");
    crate::scheduling::abort();
}
