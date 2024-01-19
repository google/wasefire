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
use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::Range;

use wasefire_interpreter::{
    Call, Error, InstId, Module, RunResult, Store as InterpreterStore, Val,
};
use wasefire_logger as log;

use super::{MemoryApi, StoreApi};
use crate::Trap;

#[derive(Debug, Default)]
pub struct Store(InterpreterStore<'static>);

impl Store {
    pub fn instantiate(
        &mut self, module: Module<'static>, memory: &'static mut [u8],
    ) -> Result<InstId, Error> {
        self.0.instantiate(module, memory)
    }

    pub fn link_func(
        &mut self, module: &'static str, name: &'static str, params: usize, results: usize,
    ) -> Result<(), Error> {
        self.0.link_func(module, name, params, results)
    }

    pub fn link_func_default(&mut self, module: &'static str) -> Result<(), Error> {
        self.0.link_func_default(module)
    }

    pub fn invoke<'a>(
        &'a mut self, inst: InstId, name: &str, args: Vec<Val>,
    ) -> Result<RunResult<'a, 'static>, Error> {
        self.0.invoke(inst, name, args)
    }

    pub fn last_call(&mut self) -> Option<Call<'_, 'static>> {
        self.0.last_call()
    }
}

impl StoreApi for Store {
    type Memory<'a> = Memory<'a>
    where Self: 'a;

    fn memory(&mut self) -> Memory<'_> {
        Memory::new(&mut self.0)
    }
}

// TODO: This could be a slice-cell crate. And should probably already be exposed in the
// interpreter?
pub struct Memory<'a> {
    store: *mut InterpreterStore<'static>,
    lifetime: PhantomData<&'a ()>,
    // Sorted ranges of borrow.
    borrow: RefCell<Vec<(bool, Range<usize>)>>,
}

impl<'a> Memory<'a> {
    fn new(store: &'a mut InterpreterStore<'static>) -> Self {
        Self { store, lifetime: PhantomData, borrow: RefCell::new(Vec::new()) }
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn data(&self) -> &mut [u8] {
        unsafe { &mut *self.store }.last_call().unwrap().mem()
    }

    fn borrow(&self, range: Range<usize>) -> Result<(), Trap> {
        self.borrow_impl((false, range))
    }

    fn borrow_mut(&self, range: Range<usize>) -> Result<(), Trap> {
        self.borrow_impl((true, range))
    }

    /// Returns whether the borrow is allowed.
    fn borrow_impl(&self, y: (bool, Range<usize>)) -> Result<(), Trap> {
        let mut borrow = self.borrow.borrow_mut();
        let i = match borrow.iter().position(|x| y.1.start < x.1.end) {
            None => {
                borrow.push(y);
                return Ok(());
            }
            Some(x) => x,
        };
        let j = match borrow[i ..].iter().position(|x| y.1.end <= x.1.start) {
            None => borrow.len(),
            Some(x) => i + x,
        };
        if i == j {
            borrow.insert(i, y);
            return Ok(());
        }
        if y.0 || borrow[i .. j].iter().any(|x| x.0) {
            return Err(Trap);
        }
        borrow[i].1.start = core::cmp::min(borrow[i].1.start, y.1.start);
        borrow[i].1.end = core::cmp::max(borrow[j - 1].1.end, y.1.end);
        borrow.drain(i + 1 .. j);
        Ok(())
    }
}

impl<'a> MemoryApi for Memory<'a> {
    fn get(&self, ptr: u32, len: u32) -> Result<&[u8], Trap> {
        let ptr = ptr as usize;
        let len = len as usize;
        let range = ptr .. ptr.checked_add(len).ok_or(Trap)?;
        self.borrow(range.clone())?;
        <[u8]>::get(unsafe { self.data() }, range).ok_or(Trap)
    }

    fn get_mut(&self, ptr: u32, len: u32) -> Result<&mut [u8], Trap> {
        let ptr = ptr as usize;
        let len = len as usize;
        let range = ptr .. ptr.checked_add(len).ok_or(Trap)?;
        self.borrow_mut(range.clone())?;
        let data = unsafe { self.data() };
        let data = unsafe { core::slice::from_raw_parts_mut(data.as_ptr() as *mut u8, data.len()) };
        <[u8]>::get_mut(data, range).ok_or(Trap)
    }

    fn alloc(&mut self, size: u32, align: u32) -> Result<u32, Trap> {
        self.borrow.borrow_mut().clear();
        let store = unsafe { &mut *self.store };
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
    }
}
