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

//! Applet interface.

use wasefire_protocol::applet::ExitStatus;

use crate::{Error, Trap};

/// Applet interface.
pub trait Api: Send {
    /// Installs or uninstalls an applet.
    ///
    /// An empty transfer uninstalls the applet (because valid applets are not empty). A non-empty
    /// transfer installs the transferred applet.
    ///
    /// Calling `start(true)` will uninstall the current applet if any. It is always possible to
    /// call `start()`, in which case a new transfer process is started.
    ///
    /// Until `finish()` is called, there should be no installed applet. In particular, if the
    /// platform reboots and `get()` is called, the returned slice should be empty.
    type Install: crate::transfer::Api;

    /// Returns the persisted applet.
    ///
    /// # Safety
    ///
    /// This function must not be called during a transfer process (unless it's a dry-run). Besides,
    /// whenever `start(true)` is called, it invalidates the result of a previous call to this
    /// function.
    unsafe fn get() -> Result<&'static [u8], Error>;

    /// Notifies an applet start.
    fn notify_start() {}

    /// Notifies an applet exit.
    fn notify_exit(status: ExitStatus) {
        let _ = status;
    }
}

/// Applet install interface.
pub type Install<B> = <super::Applet<B> as Api>::Install;

/// Interface to an applet memory.
pub trait Memory {
    /// Returns a given shared slice from the applet memory.
    fn get(&self, ptr: u32, len: u32) -> Result<&[u8], Trap>;

    /// Returns a given exclusive slice from the applet memory.
    #[allow(clippy::mut_from_ref)]
    fn get_mut(&self, ptr: u32, len: u32) -> Result<&mut [u8], Trap>;

    /// Allocates in the applet memory (traps in case of errors).
    ///
    /// This function always returns a valid allocated pointer. If the size is zero or there is not
    /// enough memory, a trap is returned.
    fn alloc(&mut self, size: u32, align: u32) -> Result<u32, Trap>;
}

/// Helper methods on top of `Memory`.
pub trait MemoryExt: Memory {
    /// Returns a shared slice from the applet memory or `None` if `ptr == 0`.
    fn get_opt(&self, ptr: u32, len: u32) -> Result<Option<&[u8]>, Trap> {
        Ok(match ptr {
            0 => None,
            _ => Some(self.get(ptr, len)?),
        })
    }

    /// Returns a shared array from the applet memory.
    fn get_array<const LEN: usize>(&self, ptr: u32) -> Result<&[u8; LEN], Trap> {
        self.get(ptr, LEN as u32).map(|x| x.try_into().unwrap())
    }

    /// Returns an exclusive array from the applet memory.
    fn get_array_mut<const LEN: usize>(&self, ptr: u32) -> Result<&mut [u8; LEN], Trap> {
        self.get_mut(ptr, LEN as u32).map(|x| x.try_into().unwrap())
    }

    /// Returns a shared object from the applet memory.
    #[allow(clippy::wrong_self_convention)]
    fn from_bytes<T: bytemuck::AnyBitPattern>(&self, ptr: u32) -> Result<&T, Trap> {
        bytemuck::try_from_bytes(self.get(ptr, core::mem::size_of::<T>() as u32)?).map_err(|_| Trap)
    }

    /// Returns an exclusive object from the applet memory.
    #[allow(clippy::wrong_self_convention)]
    fn from_bytes_mut<T: bytemuck::NoUninit + bytemuck::AnyBitPattern>(
        &self, ptr: u32,
    ) -> Result<&mut T, Trap> {
        bytemuck::try_from_bytes_mut(self.get_mut(ptr, core::mem::size_of::<T>() as u32)?)
            .map_err(|_| Trap)
    }

    /// Allocates a copy of `data` unless it's empty.
    ///
    /// Always returns `data.len()`. Also writes it to `len_ptr` if provided.
    fn alloc_copy(&mut self, ptr_ptr: u32, len_ptr: Option<u32>, data: &[u8]) -> Result<u32, Trap> {
        let len = data.len() as u32;
        if 0 < len {
            let ptr = self.alloc(len, 1)?;
            self.get_mut(ptr, len)?.copy_from_slice(data);
            self.get_mut(ptr_ptr, 4)?.copy_from_slice(&ptr.to_le_bytes());
        }
        if let Some(len_ptr) = len_ptr {
            self.get_mut(len_ptr, 4)?.copy_from_slice(&len.to_le_bytes());
        }
        Ok(len)
    }
}

impl<T: Memory> MemoryExt for T {}

/// Interface to the event handlers of an applet.
#[cfg(feature = "api-vendor")]
pub trait Handlers<Key> {
    /// Registers an event handler for a given key.
    ///
    /// The key must not exist, otherwise the applet will trap.
    fn register(&mut self, key: Key, func: u32, data: u32) -> Result<(), Trap>;

    /// Unregisters the event handler of a given key.
    ///
    /// The key must exist, otherwise the applet will trap.
    fn unregister(&mut self, key: Key) -> Result<(), Trap>;
}
