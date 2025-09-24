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

use alloc::vec;
use alloc::vec::Vec;

use wasefire_board_api::applet::Memory as AppletMemory;
use wasefire_interpreter::{
    Call, Error, InstId, Module, RunResult, Store as InterpreterStore, Val,
};
use wasefire_logger as log;
use wasefire_slice_cell::SliceCell;

use super::StoreApi;
use crate::Trap;

#[derive(Debug, Default)]
pub struct Store {
    inst: Option<InstId>,
    store: InterpreterStore<'static>,
}

impl Store {
    pub fn instantiate(
        &mut self, module: Module<'static>, memory: &'static mut [u8],
    ) -> Result<InstId, Error> {
        // We assume a single module per applet.
        assert!(self.inst.is_none());
        let inst = self.store.instantiate(module, memory)?;
        self.inst = Some(inst);
        Ok(inst)
    }

    pub fn link_func(
        &mut self, module: &'static str, name: &'static str, params: usize, results: usize,
    ) -> Result<(), Error> {
        self.store.link_func(module, name, params, results)
    }

    pub fn link_func_default(&mut self, module: &'static str) -> Result<(), Error> {
        self.store.link_func_default(module)
    }

    pub fn invoke<'a>(
        &'a mut self, inst: InstId, name: &str, args: Vec<Val>,
    ) -> Result<RunResult<'a, 'static>, Error> {
        self.store.invoke(inst, name, args)
    }

    pub fn last_call(&mut self) -> Option<Call<'_, 'static>> {
        self.store.last_call()
    }
}

impl StoreApi for Store {
    type Memory<'a>
        = Memory<'a>
    where Self: 'a;

    fn memory<'a>(&'a mut self) -> Memory<'a> {
        let inst = self.inst.unwrap();
        let store = self as *mut Store;
        let memory = SliceCell::new(self.store.memory(inst).unwrap());
        Memory { store, memory }
    }
}

pub struct Memory<'a> {
    store: *mut Store,
    memory: SliceCell<'a, u8>,
}

impl<'a> Memory<'a> {
    fn with_store<T>(&mut self, f: impl FnOnce(&mut Store) -> T) -> T {
        self.memory.reset(); // only to free the internal state early
        let store = unsafe { &mut *self.store };
        let result = f(store);
        *self = store.memory();
        result
    }
}

impl AppletMemory for Memory<'_> {
    fn get(&self, ptr: u32, len: u32) -> Result<&[u8], Trap> {
        let ptr = ptr as usize;
        let len = len as usize;
        let range = ptr .. ptr.checked_add(len).ok_or(Trap)?;
        self.memory.get_range(range).map_err(|_| Trap)
    }

    fn get_mut(&self, ptr: u32, len: u32) -> Result<&mut [u8], Trap> {
        let ptr = ptr as usize;
        let len = len as usize;
        let range = ptr .. ptr.checked_add(len).ok_or(Trap)?;
        self.memory.get_range_mut(range).map_err(|_| Trap)
    }

    fn alloc(&mut self, size: u32, align: u32) -> Result<u32, Trap> {
        self.with_store(|store| {
            let args = vec![Val::I32(size), Val::I32(align)];
            let inst = store.last_call().unwrap().inst();
            // TODO: We should ideally account this to the applet performance time.
            let result = match store.invoke(inst, "alloc", args) {
                Ok(RunResult::Done(x)) if x.len() == 1 => x[0],
                Ok(RunResult::Done(_)) => return Err(Trap),
                Ok(RunResult::Host(_)) => log::panic!("alloc called into host"),
                Err(Error::Trap) => return Err(Trap),
                Err(x) => log::panic!("alloc failed with {}", log::Debug2Format(&x)),
            };
            match result {
                Val::I32(x) if x != 0 => Ok(x),
                _ => Err(Trap),
            }
        })
    }
}
