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

#![no_std]
#![feature(never_type)]
#![feature(try_blocks)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::collections::VecDeque;
use alloc::vec::Vec;
#[cfg(feature = "native")]
use core::ffi::CStr;
use core::marker::PhantomData;

use derivative::Derivative;
use event::Key;
use wasefire_applet_api::{self as api, Api, ArrayU32, Dispatch, Id, Signature, U32};
#[cfg(feature = "board-api-storage")]
use wasefire_board_api::Singleton;
#[cfg(feature = "board-api-timer")]
use wasefire_board_api::Support;
use wasefire_board_api::{self as board, Api as Board};
use wasefire_error::Error;
#[cfg(feature = "wasm")]
use wasefire_interpreter::{self as interpreter, Call, Module, RunAnswer, Val};
use wasefire_logger as log;
#[cfg(feature = "board-api-storage")]
use wasefire_store as store;

use crate::applet::store::{Memory, Store, StoreApi};
use crate::applet::{Applet, EventAction};
use crate::event::InstId;

mod applet;
mod call;
mod event;
#[cfg(feature = "native")]
mod native;
#[cfg(feature = "internal-debug")]
mod perf;
#[cfg(feature = "board-api-platform-protocol")]
mod protocol;

#[cfg(all(feature = "native", not(target_pointer_width = "32")))]
compile_error!("Only 32-bits architectures support native applets.");

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
            log::trace!("Merging {}", log::Debug2Format(&event));
        } else if self.0.len() < MAX_EVENTS {
            log::debug!("Pushing {}", log::Debug2Format(&event));
            self.0.push_back(event);
        } else {
            log::warn!("Dropping {}", log::Debug2Format(&event));
        }
    }

    pub fn pop(&mut self) -> Option<board::Event<B>> {
        self.0.pop_front().inspect(|event| log::debug!("Popping {}", log::Debug2Format(&event)))
    }
}

pub struct Scheduler<B: Board> {
    #[cfg(feature = "board-api-storage")]
    store: store::Store<B::Storage>,
    host_funcs: Vec<Api<Id>>,
    applet: Applet<B>,
    #[cfg(feature = "board-api-timer")]
    timers: Vec<Option<Timer>>,
    #[cfg(feature = "internal-debug")]
    perf: perf::Perf<B>,
    #[cfg(feature = "board-api-platform-protocol")]
    protocol: protocol::State,
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

struct SchedulerCallT<'a, B: Board> {
    scheduler: &'a mut Scheduler<B>,
    #[cfg(feature = "wasm")]
    args: Vec<u32>,
    #[cfg(feature = "native")]
    params: &'a [u32],
    #[cfg(feature = "native")]
    result: &'a mut i32,
}

impl<'a, B: Board> core::fmt::Debug for SchedulerCallT<'a, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SchedulerCallT").finish()
    }
}

struct SchedulerCall<'a, B: Board, T> {
    erased: SchedulerCallT<'a, B>,
    phantom: PhantomData<T>,
}

impl<'a, B: Board, T> core::fmt::Debug for SchedulerCall<'a, B, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SchedulerCall").finish()
    }
}

struct DispatchSchedulerCall<'a, B> {
    phantom: PhantomData<&'a B>,
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
    fn read(&self) -> T::Params {
        #[cfg(feature = "wasm")]
        let params = &self.erased.args;
        #[cfg(feature = "native")]
        let params = self.erased.params;
        *<T::Params as ArrayU32>::from(params)
    }

    #[cfg_attr(not(feature = "board-api-button"), allow(dead_code))]
    fn inst(&mut self) -> InstId {
        #[cfg(feature = "wasm")]
        let id = self.call().inst();
        #[cfg(feature = "native")]
        let id = InstId;
        id
    }

    fn memory(&mut self) -> Memory<'_> {
        self.store().memory()
    }

    fn scheduler(&mut self) -> &mut Scheduler<B> {
        self.erased.scheduler
    }

    fn reply(self, result: Result<impl Into<U32<T::Result>>, Failure>) {
        self.reply_(result.map(|x| x.into()))
    }

    fn reply_(self, result: Result<U32<T::Result>, Failure>) {
        let result = Error::encode(match result {
            Ok(x) => Ok(*x),
            Err(Failure::Error(e)) => Err(e),
            Err(Failure::Trap(Trap)) => applet_trapped::<B>(Some(T::NAME)),
        });
        #[cfg(feature = "wasm")]
        {
            let mut this = self;
            let results = [Val::I32(result as u32)];
            #[cfg(feature = "internal-debug")]
            this.scheduler().perf.record(perf::Slot::Platform);
            let answer = this.call().resume(&results).map(|x| x.forget());
            #[cfg(feature = "internal-debug")]
            this.scheduler().perf.record(perf::Slot::Applets);
            this.erased.scheduler.process_answer(answer);
        }
        #[cfg(feature = "native")]
        {
            *self.erased.result = result;
        }
    }

    fn applet(&mut self) -> &mut Applet<B> {
        &mut self.erased.scheduler.applet
    }

    fn store(&mut self) -> &mut Store {
        self.applet().store_mut()
    }

    #[cfg(feature = "wasm")]
    fn call(&mut self) -> Call<'_, 'static> {
        self.store().last_call().unwrap()
    }
}

impl<B: Board> Scheduler<B> {
    #[cfg(feature = "wasm")]
    pub fn run(wasm: &'static [u8]) -> ! {
        let mut scheduler = Self::new();
        log::debug!("Loading applet.");
        scheduler.load(wasm);
        loop {
            scheduler.flush_events();
            scheduler.process_applet();
        }
    }

    #[cfg(feature = "native")]
    pub fn run() -> ! {
        native::set_scheduler(Self::new());
        extern "C" {
            fn applet_init();
            fn applet_main();
        }
        #[cfg(feature = "internal-debug")]
        native::with_scheduler(|x| x.perf_record(perf::Slot::Platform));
        log::debug!("Execute init.");
        unsafe { applet_init() };
        log::debug!("Execute main.");
        unsafe { applet_main() };
        #[cfg(feature = "internal-debug")]
        native::with_scheduler(|x| x.perf_record(perf::Slot::Applets));
        log::debug!("Returned from main, executing callbacks only.");
        loop {
            native::with_scheduler(|scheduler| {
                scheduler.flush_events();
                scheduler.process_event();
            });
            native::execute_callback();
        }
    }

    #[cfg(feature = "native")]
    fn dispatch(&mut self, link: &CStr, params: *const u32) -> isize {
        let name = link.to_str().unwrap();
        let index = match self.host_funcs.binary_search_by_key(&name, |x| x.descriptor().name) {
            Ok(x) => x,
            Err(_) => {
                let error = Error::world(wasefire_error::Code::NotImplemented);
                return Error::encode(Err(error)) as isize;
            }
        };
        let api_id = self.host_funcs[index].id();
        let desc = api_id.descriptor();
        let params = unsafe { core::slice::from_raw_parts(params, desc.params) };
        let mut result = 0;
        let erased = SchedulerCallT { scheduler: self, params, result: &mut result };
        let call = api_id.merge(erased);
        log::debug!("Calling {}", log::Debug2Format(&call.id()));
        call::process(call);
        result as isize
    }

    fn new() -> Self {
        let mut host_funcs = Vec::new();
        Api::<Id>::iter(&mut host_funcs, |x| x);
        host_funcs.sort_by_key(|x| x.descriptor().name);
        assert!(host_funcs.windows(2).all(|x| x[0].descriptor().name != x[1].descriptor().name));
        #[cfg(feature = "wasm")]
        let applet = {
            let mut applet = Applet::default();
            let store = applet.store_mut();
            for f in &host_funcs {
                let d = f.descriptor();
                store.link_func("env", d.name, d.params, 1).unwrap();
            }
            store.link_func_default("env").unwrap();
            applet
        };
        #[cfg(feature = "board-api-platform-protocol")]
        protocol::enable::<B>();
        #[cfg(feature = "native")]
        let applet = Applet::default();
        Self {
            #[cfg(feature = "board-api-storage")]
            store: store::Store::new(board::Storage::<B>::take().unwrap()).ok().unwrap(),
            host_funcs,
            applet,
            #[cfg(feature = "board-api-timer")]
            timers: alloc::vec![None; board::Timer::<B>::SUPPORT],
            #[cfg(feature = "internal-debug")]
            perf: perf::Perf::default(),
            #[cfg(feature = "board-api-platform-protocol")]
            protocol: protocol::State::default(),
        }
    }

    #[cfg(feature = "wasm")]
    fn load(&mut self, wasm: &'static [u8]) {
        const MEMORY_SIZE: usize = memory_size();
        #[repr(align(16))]
        struct Memory([u8; MEMORY_SIZE]);
        static mut MEMORY: Memory = Memory([0; MEMORY_SIZE]);
        #[cfg(not(feature = "unsafe-skip-validation"))]
        let module = Module::new(wasm).unwrap();
        // SAFETY: The module is valid by the feature invariant.
        #[cfg(feature = "unsafe-skip-validation")]
        let module = unsafe { Module::new_unchecked(wasm) };
        let store = self.applet.store_mut();
        // SAFETY: This function is called once in `run()`.
        let inst = store.instantiate(module, unsafe { &mut MEMORY.0 }).unwrap();
        #[cfg(feature = "internal-debug")]
        self.perf.record(perf::Slot::Platform);
        self.call(inst, "init", &[]);
        while let Some(call) = self.applet.store_mut().last_call() {
            match self.host_funcs[call.index()].descriptor().name {
                "dp" => (),
                x => log::panic!("init called {} into host", log::Debug2Format(&x)),
            }
            self.process_applet();
        }
        assert!(matches!(self.applet.pop(), EventAction::Reply));
        #[cfg(feature = "internal-debug")]
        self.perf.record(perf::Slot::Applets);
        self.call(inst, "main", &[]);
    }

    fn flush_events(&mut self) {
        while let Some(event) = B::try_event() {
            self.triage_event(event);
        }
    }

    fn triage_event(&mut self, event: board::Event<B>) {
        #[cfg(feature = "board-api-platform-protocol")]
        if protocol::should_process_event(&event) {
            protocol::process_event(self, event);
            return;
        }
        self.applet.push(event);
    }

    /// Returns whether execution should resume.
    fn process_event(&mut self) -> bool {
        let event = loop {
            match self.applet.pop() {
                EventAction::Handle(event) => break event,
                EventAction::Wait => {
                    #[cfg(feature = "internal-debug")]
                    self.perf.record(perf::Slot::Platform);
                    let event = B::wait_event();
                    #[cfg(feature = "internal-debug")]
                    self.perf.record(perf::Slot::Waiting);
                    self.triage_event(event);
                }
                #[cfg(feature = "wasm")]
                EventAction::Reply => return true,
            }
        };
        event::process(self, event);
        false
    }

    #[cfg(feature = "wasm")]
    fn process_applet(&mut self) {
        let call = match self.applet.store_mut().last_call() {
            Some(x) => x,
            None => {
                self.process_event();
                return;
            }
        };
        let api_id = match self.host_funcs.get(call.index()) {
            Some(x) => x.id(),
            None => {
                let error = Error::encode(Err(Error::world(wasefire_error::Code::NotImplemented)));
                let answer = call.resume(&[Val::I32(error as u32)]).map(|x| x.forget());
                self.process_answer(answer);
                return;
            }
        };
        let args = call.args();
        debug_assert_eq!(args.len(), api_id.descriptor().params);
        let args = args.iter().map(|x| x.unwrap_i32()).collect();
        let erased = SchedulerCallT { scheduler: self, args };
        let call = api_id.merge(erased);
        log::debug!("Calling {}", log::Debug2Format(&call.id()));
        call::process(call);
    }

    #[allow(dead_code)] // in case there are no events
    fn disable_event(&mut self, key: Key<B>) -> Result<(), Trap> {
        self.applet.disable(key)?;
        self.flush_events();
        Ok(())
    }

    #[cfg(feature = "wasm")]
    fn call(&mut self, inst: InstId, name: &str, args: &[u32]) {
        log::debug!("Schedule thread {}{:?}.", name, args);
        let args = args.iter().map(|&x| Val::I32(x)).collect();
        #[cfg(feature = "internal-debug")]
        self.perf.record(perf::Slot::Platform);
        let answer = self.applet.store_mut().invoke(inst, name, args).map(|x| x.forget());
        #[cfg(feature = "internal-debug")]
        self.perf.record(perf::Slot::Applets);
        self.process_answer(answer);
    }

    #[cfg(feature = "wasm")]
    fn process_answer(&mut self, result: Result<RunAnswer, interpreter::Error>) {
        match result {
            Ok(RunAnswer::Done(x)) => {
                log::debug!("Thread is done.");
                debug_assert!(x.is_empty());
                self.applet.done();
            }
            Ok(RunAnswer::Host) => (),
            Err(interpreter::Error::Trap) => applet_trapped::<B>(None),
            Err(e) => log::panic!("{}", log::Debug2Format(&e)),
        }
    }
}

fn applet_trapped<B: Board>(reason: Option<&'static str>) -> ! {
    // Until we support multiple applets, we just exit the platform when the applet traps.
    match reason {
        None => log::error!("Applet trapped in wasm (think segfault)."),
        Some("sa") => log::error!("Applet aborted (probably a panic)."),
        Some(name) => log::error!("Applet trapped calling host {:?}.", name),
    }
    <board::Debug<B> as board::debug::Api>::exit(false);
}

struct Trap;

enum Failure {
    Trap(Trap),
    Error(Error),
}

impl From<Trap> for Failure {
    fn from(value: Trap) -> Self {
        Self::Trap(value)
    }
}

impl From<Error> for Failure {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

#[cfg(feature = "wasm")]
const fn memory_size() -> usize {
    let page = match option_env!("WASEFIRE_MEMORY_PAGE_COUNT") {
        Some(x) => {
            let x = x.as_bytes();
            assert!(x.len() == 1, "not a single digit");
            let x = x[0];
            assert!(x.is_ascii_digit(), "not a single digit");
            (x - b'0') as usize
        }
        None => 1,
    };
    page * 0x10000
}
