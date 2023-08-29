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

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(result_option_inspect)]
#![feature(try_blocks)]

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::Range;

use bytemuck::{AnyBitPattern, NoUninit};
use derivative::Derivative;
use event::Key;
use stores::{Applet, EventAction};
use wasefire_applet_api::{self as api, Api, ArrayU32, Dispatch, Id, Signature};
use wasefire_board_api::{self as board, Api as Board, Singleton, Support};
use wasefire_interpreter::{
    self as interpreter, Call, Error, InstId, Module, RunAnswer, RunResult, Store, Val,
};
use wasefire_logger::{self as logger, *};
use wasefire_store as store;

mod call;
mod event;
#[cfg(feature = "debug")]
mod perf;
mod stores;

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Events<B: Board>(VecDeque<board::Event<B>>);

impl<B: Board> Events<B> {
    pub const fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, event: board::Event<B>) {
        const MAX_EVENTS: usize = 10;
        if self.0.contains(&event) {
            trace!("Merging {}", Debug2Format(&event));
        } else if self.0.len() < MAX_EVENTS {
            debug!("Pushing {}", Debug2Format(&event));
            self.0.push_back(event);
        } else {
            warn!("Dropping {}", Debug2Format(&event));
        }
    }

    pub fn pop(&mut self) -> Option<board::Event<B>> {
        self.0.pop_front().inspect(|event| debug!("Popping {}", Debug2Format(&event)))
    }
}

pub struct Scheduler<B: Board> {
    store: store::Store<B::Storage>,
    host_funcs: Vec<Api<Id>>,
    applet: Applet<B>,
    timers: Vec<Option<Timer>>,
    #[cfg(feature = "debug")]
    perf: perf::Perf<B>,
}

#[derive(Clone)]
struct Timer {
    // TODO: Add AppletId.
}

impl<B: Board> core::fmt::Debug for Scheduler<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Scheduler").finish()
    }
}

pub struct SchedulerCallT<'a, B: Board> {
    scheduler: &'a mut Scheduler<B>,
    args: Vec<u32>,
}

impl<'a, B: Board> core::fmt::Debug for SchedulerCallT<'a, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SchedulerCallT").finish()
    }
}

pub struct SchedulerCall<'a, B: Board, T> {
    erased: SchedulerCallT<'a, B>,
    phantom: PhantomData<T>,
}

impl<'a, B: Board, T> core::fmt::Debug for SchedulerCall<'a, B, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SchedulerCall").finish()
    }
}

pub struct DispatchSchedulerCall<'a, B> {
    phantom: PhantomData<&'a B>,
}

pub enum SchedulerResult<'a, B: Board> {
    Call(Api<DispatchSchedulerCall<'a, B>>),
    Yield,
    Wait,
}

impl<'a, B: Board> Dispatch for DispatchSchedulerCall<'a, B> {
    type Erased = SchedulerCallT<'a, B>;
    type Merged<T: api::Signature> = SchedulerCall<'a, B, T>;

    fn merge<T: api::Signature>(erased: Self::Erased) -> Self::Merged<T> {
        SchedulerCall { erased, phantom: PhantomData }
    }

    fn erase<T: api::Signature>(merged: Self::Merged<T>) -> Self::Erased {
        merged.erased
    }
}

impl<'a, B: Board, T: Signature> SchedulerCall<'a, B, T> {
    pub fn read(&self) -> T::Params {
        *<T::Params as ArrayU32>::from(&self.erased.args)
    }

    pub fn inst(&mut self) -> InstId {
        self.call().inst()
    }

    pub fn memory(&mut self) -> Memory {
        Memory::new(self.store())
    }

    pub fn scheduler(&mut self) -> &mut Scheduler<B> {
        self.erased.scheduler
    }

    pub fn reply(mut self, results: Result<T::Results, Trap>) {
        match results.map(convert_results::<T>) {
            Ok(results) => {
                #[cfg(feature = "debug")]
                self.scheduler().perf.record(perf::Slot::Platform);
                let answer = self.call().resume(&results).map(|x| x.forget());
                #[cfg(feature = "debug")]
                self.scheduler().perf.record(perf::Slot::Applets);
                self.erased.scheduler.process_answer(answer);
            }
            Err(Trap) => logger::panic!("Applet trapped in host."),
        }
    }

    fn applet(&mut self) -> &mut Applet<B> {
        &mut self.erased.scheduler.applet
    }

    fn store(&mut self) -> &mut Store<'static> {
        self.applet().store_mut()
    }

    fn call(&mut self) -> Call<'_, 'static> {
        self.store().last_call().unwrap()
    }
}

impl<B: Board> Scheduler<B> {
    pub fn run(wasm: &'static [u8]) -> ! {
        let mut scheduler = Self::new();
        debug!("Loading applet.");
        scheduler.load(wasm);
        loop {
            scheduler.flush_events();
            scheduler.process_applet();
        }
    }

    fn new() -> Self {
        let mut host_funcs = Vec::new();
        Api::<Id>::iter(&mut host_funcs, |x| x);
        host_funcs.sort_by_key(|x| x.descriptor().name);
        assert!(host_funcs.windows(2).all(|x| x[0].descriptor().name != x[1].descriptor().name));
        let mut applet = Applet::default();
        let store = applet.store_mut();
        for f in &host_funcs {
            let d = f.descriptor();
            store.link_func("env", d.name, d.params, d.results).unwrap();
        }
        Self {
            store: store::Store::new(board::Storage::<B>::take().unwrap()).ok().unwrap(),
            host_funcs,
            applet,
            timers: vec![None; board::Timer::<B>::SUPPORT],
            #[cfg(feature = "debug")]
            perf: perf::Perf::default(),
        }
    }

    fn load(&mut self, wasm: &'static [u8]) {
        #[repr(align(16))]
        struct Memory([u8; 0x10000]);
        static mut MEMORY: Memory = Memory([0; 0x10000]);
        #[cfg(not(feature = "unsafe-skip-validation"))]
        let module = Module::new(wasm).unwrap();
        // SAFETY: The module is valid by the feature invariant.
        #[cfg(feature = "unsafe-skip-validation")]
        let module = unsafe { Module::new_unchecked(wasm) };
        let store = self.applet.store_mut();
        // SAFETY: This function is called once in `run()`.
        let inst = store.instantiate(module, unsafe { &mut MEMORY.0 }).unwrap();
        #[cfg(feature = "debug")]
        self.perf.record(perf::Slot::Platform);
        match store.invoke(inst, "init", vec![]) {
            Ok(RunResult::Done(x)) => assert!(x.is_empty()),
            Ok(RunResult::Host { .. }) => logger::panic!("init called into host"),
            Err(Error::NotFound) => (),
            Err(e) => core::panic!("{e:?}"),
        }
        #[cfg(feature = "debug")]
        self.perf.record(perf::Slot::Applets);
        self.call(inst, "main", &[]);
    }

    fn flush_events(&mut self) {
        while let Some(event) = B::try_event() {
            self.applet.push(event);
        }
    }

    /// Returns whether execution should resume.
    fn process_event(&mut self) -> bool {
        let event = loop {
            match self.applet.pop() {
                EventAction::Handle(event) => break event,
                EventAction::Wait => {
                    #[cfg(feature = "debug")]
                    self.perf.record(perf::Slot::Platform);
                    let event = B::wait_event();
                    #[cfg(feature = "debug")]
                    self.perf.record(perf::Slot::Waiting);
                    self.applet.push(event);
                }
                EventAction::Reply => return true,
            }
        };
        event::process(self, event);
        false
    }

    fn process_applet(&mut self) {
        let call = match self.applet.store_mut().last_call() {
            Some(x) => x,
            None => {
                self.process_event();
                return;
            }
        };
        let api_id = self.host_funcs[call.index()].id();
        let args = call.args();
        debug_assert_eq!(args.len(), api_id.descriptor().params);
        let args = args.iter().map(|x| x.unwrap_i32()).collect();
        let erased = SchedulerCallT { scheduler: self, args };
        let call = api_id.merge(erased);
        debug!("Calling {}", Debug2Format(&call.id()));
        call::process(call);
    }

    fn disable_event(&mut self, key: Key<B>) -> Result<(), Trap> {
        self.applet.disable(key)?;
        self.flush_events();
        Ok(())
    }

    fn call(&mut self, inst: InstId, name: &'static str, args: &[u32]) {
        debug!("Schedule thread {}{:?}.", name, args);
        let args = args.iter().map(|&x| Val::I32(x)).collect();
        #[cfg(feature = "debug")]
        self.perf.record(perf::Slot::Platform);
        let answer = self.applet.store_mut().invoke(inst, name, args).map(|x| x.forget());
        #[cfg(feature = "debug")]
        self.perf.record(perf::Slot::Applets);
        self.process_answer(answer);
    }

    fn process_answer(&mut self, result: Result<RunAnswer, interpreter::Error>) {
        match result {
            Ok(RunAnswer::Done(x)) => {
                debug!("Thread is done.");
                debug_assert!(x.is_empty());
                self.applet.done();
            }
            Ok(RunAnswer::Host) => (),
            Err(Error::Trap) => logger::panic!("Applet trapped in wasm."),
            Err(e) => core::panic!("{e:?}"),
        }
    }
}

fn convert_results<T: Signature>(results: T::Results) -> Vec<Val> {
    <T::Results as ArrayU32>::into(&results).iter().map(|&x| Val::I32(x)).collect()
}

pub struct Trap;

impl From<()> for Trap {
    fn from(_: ()) -> Self {
        Trap
    }
}

// TODO: This could be a slice-cell crate. And should probably already be exposed in the
// interpreter?
pub struct Memory<'a> {
    store: *mut Store<'static>,
    lifetime: PhantomData<&'a ()>,
    // Sorted ranges of borrow.
    borrow: RefCell<Vec<(bool, Range<usize>)>>,
}

impl<'a> Memory<'a> {
    fn new(store: &'a mut Store<'static>) -> Self {
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

    pub fn get(&self, ptr: u32, len: u32) -> Result<&[u8], Trap> {
        let ptr = ptr as usize;
        let len = len as usize;
        let range = ptr .. ptr.checked_add(len).ok_or(Trap)?;
        self.borrow(range.clone())?;
        <[u8]>::get(unsafe { self.data() }, range).ok_or(Trap)
    }

    pub fn get_opt(&self, ptr: u32, len: u32) -> Result<Option<&[u8]>, Trap> {
        Ok(match ptr {
            0 => None,
            _ => Some(self.get(ptr, len)?),
        })
    }

    pub fn get_array<const LEN: usize>(&self, ptr: u32) -> Result<&[u8; LEN], Trap> {
        self.get(ptr, LEN as u32).map(|x| x.try_into().unwrap())
    }

    pub fn get_mut(&self, ptr: u32, len: u32) -> Result<&mut [u8], Trap> {
        let ptr = ptr as usize;
        let len = len as usize;
        let range = ptr .. ptr.checked_add(len).ok_or(Trap)?;
        self.borrow_mut(range.clone())?;
        let data = unsafe { self.data() };
        let data = unsafe { core::slice::from_raw_parts_mut(data.as_ptr() as *mut u8, data.len()) };
        <[u8]>::get_mut(data, range).ok_or(Trap)
    }

    pub fn get_array_mut<const LEN: usize>(&self, ptr: u32) -> Result<&mut [u8; LEN], Trap> {
        self.get_mut(ptr, LEN as u32).map(|x| x.try_into().unwrap())
    }

    pub fn from_bytes<T: AnyBitPattern>(&self, ptr: u32) -> Result<&T, Trap> {
        Ok(bytemuck::from_bytes(self.get(ptr, core::mem::size_of::<T>() as u32)?))
    }

    pub fn from_bytes_mut<T: NoUninit + AnyBitPattern>(&self, ptr: u32) -> Result<&mut T, Trap> {
        Ok(bytemuck::from_bytes_mut(self.get_mut(ptr, core::mem::size_of::<T>() as u32)?))
    }

    pub fn alloc(&mut self, size: u32, align: u32) -> u32 {
        self.borrow.borrow_mut().clear();
        let store = unsafe { &mut *self.store };
        let args = vec![Val::I32(size), Val::I32(align)];
        let inst = store.last_call().unwrap().inst();
        let result: Result<Val, Error> = try {
            // TODO: We should ideally account this to the applet performance time.
            match store.invoke(inst, "alloc", args)? {
                RunResult::Done(x) if x.len() == 1 => x[0],
                _ => Err(Error::Invalid)?,
            }
        };
        match result {
            Ok(Val::I32(x)) => x,
            _ => 0,
        }
    }
}
