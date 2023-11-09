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

use bytemuck::{AnyBitPattern, NoUninit};

pub use self::impl_::Store;
use crate::Trap;

#[cfg_attr(feature = "native", path = "store/native.rs")]
#[cfg_attr(feature = "wasm", path = "store/wasm.rs")]
mod impl_;

pub type Memory<'a> = <Store as StoreApi>::Memory<'a>;

pub trait StoreApi {
    type Memory<'a>: MemoryApi
    where Self: 'a;

    fn memory(&mut self) -> Self::Memory<'_>;
}

pub trait MemoryApi {
    fn get(&self, ptr: u32, len: u32) -> Result<&[u8], Trap>;
    fn get_mut(&self, ptr: u32, len: u32) -> Result<&mut [u8], Trap>;
    fn alloc(&mut self, size: u32, align: u32) -> u32;

    fn get_opt(&self, ptr: u32, len: u32) -> Result<Option<&[u8]>, Trap> {
        Ok(match ptr {
            0 => None,
            _ => Some(self.get(ptr, len)?),
        })
    }

    fn get_array<const LEN: usize>(&self, ptr: u32) -> Result<&[u8; LEN], Trap> {
        self.get(ptr, LEN as u32).map(|x| x.try_into().unwrap())
    }

    fn get_array_mut<const LEN: usize>(&self, ptr: u32) -> Result<&mut [u8; LEN], Trap> {
        self.get_mut(ptr, LEN as u32).map(|x| x.try_into().unwrap())
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_bytes<T: AnyBitPattern>(&self, ptr: u32) -> Result<&T, Trap> {
        Ok(bytemuck::from_bytes(self.get(ptr, core::mem::size_of::<T>() as u32)?))
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_bytes_mut<T: NoUninit + AnyBitPattern>(&self, ptr: u32) -> Result<&mut T, Trap> {
        Ok(bytemuck::from_bytes_mut(self.get_mut(ptr, core::mem::size_of::<T>() as u32)?))
    }
}
