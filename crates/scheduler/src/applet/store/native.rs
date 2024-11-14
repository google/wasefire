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

use core::alloc::Layout;

use super::{MemoryApi, StoreApi};
use crate::Trap;

#[derive(Debug, Default)]
pub struct Store(());

pub struct Memory;

impl StoreApi for Store {
    type Memory<'a>
        = Memory
    where Self: 'a;

    fn memory(&mut self) -> Memory {
        Memory
    }
}

// SATEFY: For now we trust the applet. We could do some additional checks. But those already run
// when running the applet in Wasm. Native applets are meant for performance.
impl MemoryApi for Memory {
    fn get(&self, ptr: u32, len: u32) -> Result<&[u8], Trap> {
        match (ptr, len) {
            (_, 0) => Ok(&[]),
            (0, _) => Err(Trap),
            _ => Ok(unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) }),
        }
    }

    fn get_mut(&self, ptr: u32, len: u32) -> Result<&mut [u8], Trap> {
        match (ptr, len) {
            (_, 0) => Ok(&mut []),
            (0, _) => Err(Trap),
            _ => Ok(unsafe { core::slice::from_raw_parts_mut(ptr as *mut u8, len as usize) }),
        }
    }

    fn alloc(&mut self, size: u32, align: u32) -> Result<u32, Trap> {
        Ok(board_alloc(size, align) as u32)
    }
}

#[no_mangle]
extern "C" fn board_alloc(size: u32, align: u32) -> *mut u8 {
    let layout = Layout::from_size_align(size as usize, align as usize).unwrap();
    unsafe { alloc::alloc::alloc(layout) }
}

#[no_mangle]
extern "C" fn board_dealloc(ptr: *mut u8, size: u32, align: u32) {
    let layout = Layout::from_size_align(size as usize, align as usize).unwrap();
    unsafe { alloc::alloc::dealloc(ptr, layout) };
}
