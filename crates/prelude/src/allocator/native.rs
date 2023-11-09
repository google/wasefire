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

use core::alloc::{GlobalAlloc, Layout};

extern "C" {
    fn board_alloc(size: u32, align: u32) -> *mut u8;
    fn board_dealloc(ptr: *mut u8, size: u32, align: u32);
}

#[no_mangle]
extern "C" fn applet_init() {}

struct Allocator;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { board_alloc(layout.size() as u32, layout.align() as u32) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { board_dealloc(ptr, layout.size() as u32, layout.align() as u32) }
    }
}
