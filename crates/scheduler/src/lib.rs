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

use derive_where::derive_where;
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
use wasefire_one_of::exactly_one_of;
use wasefire_protocol::applet::ExitStatus;
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
mod protocol;

#[cfg(all(feature = "native", not(target_pointer_width = "32")))]
compile_error!("Only 32-bits architectures support native applets.");

exactly_one_of!["native", "wasm"];

#[derive_where(Default)]
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
            log::trace!("Merging {:?}", event);
        } else if self.0.len() < MAX_EVENTS {
            log::debug!("Pushing {:?}", event);
            self.0.push_back(event);
        } else {
            log::warn!("Dropping {:?}", event);
        }
    }

    pub fn pop(&mut self) -> Option<board::Event<B>> {
        self.0.pop_front().inspect(|event| log::debug!("Popping {:?}", event))
    }
}

pub struct Scheduler<B: Board> {
    #[cfg(feature = "board-api-storage")]
    store: store::Store<B::Storage>,
    host_funcs: Vec<Api<Id>>,
    applet: applet::Slot<B>,
    #[cfg(feature = "board-api-timer")]
    timers: Vec<Option<Timer>>,
    #[cfg(feature = "internal-debug")]
    perf: perf::Perf<B>,
    protocol: protocol::State,
}

#[cfg(feature = "board-api-timer")]
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

impl<B: Board> core::fmt::Debug for SchedulerCallT<'_, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SchedulerCallT").finish()
    }
}

struct SchedulerCall<'a, B: Board, T> {
    erased: SchedulerCallT<'a, B>,
    phantom: PhantomData<T>,
}

impl<B: Board, T> core::fmt::Debug for SchedulerCall<'_, B, T> {
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

impl<B: Board, T: Signature> SchedulerCall<'_, B, T> {
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

    fn reply_(mut self, result: Result<U32<T::Result>, Failure>) {
        let result = Error::encode(match result {
            Ok(x) => Ok(*x),
            Err(Failure::Error(e)) => Err(e),
            Err(Failure::Trap(Trap)) => {
                return applet_trapped::<B>(self.scheduler(), Some(T::NAME));
            }
        });
        #[cfg(feature = "wasm")]
        {
            let results = [Val::I32(result as u32)];
            #[cfg(feature = "internal-debug")]
            self.scheduler().perf.record(perf::Slot::Platform);
            let answer = self.call().resume(&results).map(|x| x.forget());
            #[cfg(feature = "internal-debug")]
            self.scheduler().perf.record(perf::Slot::Applets);
            self.erased.scheduler.process_answer(answer);
        }
        #[cfg(feature = "native")]
        {
            *self.erased.result = result;
        }
    }

    fn applet(&mut self) -> &mut Applet<B> {
        self.erased.scheduler.applet.get().unwrap()
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
    pub fn run() -> ! {
        let mut scheduler = Self::new();
        if let Err(error) = scheduler.start_applet() {
            log::warn!("Failed to start applet: {}", error);
        }
        loop {
            scheduler.flush_events();
            scheduler.process_applet();
        }
    }

    #[cfg(feature = "native")]
    pub fn run() -> ! {
        native::set_scheduler(Self::new());
        unsafe extern "C" {
            unsafe fn applet_init();
            unsafe fn applet_main();
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
        protocol::enable::<B>();
        Self {
            #[cfg(feature = "board-api-storage")]
            store: store::Store::new(board::Storage::<B>::take().unwrap()).ok().unwrap(),
            host_funcs,
            #[cfg(feature = "native")]
            applet: applet::Slot::Running(Applet::default()),
            #[cfg(feature = "wasm")]
            applet: applet::Slot::Empty,
            #[cfg(feature = "board-api-timer")]
            timers: alloc::vec![None; board::Timer::<B>::SUPPORT],
            #[cfg(feature = "internal-debug")]
            perf: perf::Perf::default(),
            protocol: protocol::State::default(),
        }
    }

    fn stop_applet(&mut self, status: ExitStatus) {
        let applet = match self.applet.get() {
            Some(x) => x,
            None => return,
        };
        log::info!("Stopping applet.");
        <board::Applet<B> as board::applet::Api>::notify_exit(status);
        applet.free();
        self.applet = applet::Slot::Exited(status);
        #[cfg(feature = "native")]
        {
            log::debug!("Applet stopped, executing protocol events only.");
            loop {
                let event = B::wait_event();
                if protocol::should_process_event(&event) {
                    protocol::process_event(self, event);
                }
            }
        }
    }

    #[cfg(feature = "wasm")]
    fn start_applet(&mut self) -> Result<(), Error> {
        const MEMORY_SIZE: usize = memory_size();
        #[repr(align(16))]
        struct Memory([u8; MEMORY_SIZE]);
        static mut MEMORY: Memory = Memory([0; MEMORY_SIZE]);

        // SAFETY: We stop the applet before installing a new one.
        let wasm = unsafe { <board::Applet<B> as board::applet::Api>::get()? };
        if wasm.is_empty() {
            log::info!("No applet to start.");
            return Ok(());
        }
        log::info!("Starting applet.");
        <board::Applet<B> as board::applet::Api>::notify_start();
        self.applet = applet::Slot::Running(Applet::default());
        #[cfg(not(feature = "unsafe-skip-validation"))]
        let module = Module::new(wasm)?;
        // SAFETY: The module is valid by the feature invariant.
        #[cfg(feature = "unsafe-skip-validation")]
        let module = unsafe { Module::new_unchecked(wasm) };
        let applet = self.applet.get().unwrap();
        let store = applet.store_mut();
        for f in &self.host_funcs {
            let d = f.descriptor();
            store.link_func("env", d.name, d.params, 1)?;
        }
        store.link_func_default("env")?;
        // SAFETY: We support only one applet at the moment, and the previous one is dropped when
        // assigning to self.applet above.
        #[allow(static_mut_refs)]
        let inst = store.instantiate(module, unsafe { &mut MEMORY.0 })?;
        #[cfg(feature = "internal-debug")]
        self.perf.record(perf::Slot::Platform);
        self.call(inst, "init", &[]);
        while let Some(call) = self.applet.get().unwrap().store_mut().last_call() {
            match self.host_funcs[call.index()].descriptor().name {
                "dp" => (),
                x => {
                    log::warn!("init called {} into host", log::Debug2Format(&x));
                    return Ok(applet_trapped(self, Some(x)));
                }
            }
            self.process_applet();
        }
        assert!(matches!(self.applet.get().unwrap().pop(), EventAction::Reply));
        #[cfg(feature = "internal-debug")]
        self.perf.record(perf::Slot::Applets);
        self.call(inst, "main", &[]);
        Ok(())
    }

    fn flush_events(&mut self) {
        while let Some(event) = B::try_event() {
            self.triage_event(event);
        }
    }

    fn triage_event(&mut self, event: board::Event<B>) {
        if protocol::should_process_event(&event) {
            return protocol::process_event(self, event);
        }
        match self.applet.get() {
            None => log::warn!("{:?} matches no applet", event),
            Some(applet) => applet.push(event),
        }
    }

    /// Returns whether execution should resume.
    fn process_event(&mut self) -> bool {
        let event = loop {
            let applet = match self.applet.get() {
                Some(x) => x,
                None => return false,
            };
            match applet.pop() {
                EventAction::Handle(event) => break event,
                EventAction::Wait => self.wait_event(),
                #[cfg(feature = "wasm")]
                EventAction::Reply => return true,
            }
        };
        event::process(self, event);
        false
    }

    fn wait_event(&mut self) {
        #[cfg(feature = "internal-debug")]
        self.perf.record(perf::Slot::Platform);
        let event = B::wait_event();
        #[cfg(feature = "internal-debug")]
        self.perf.record(perf::Slot::Waiting);
        self.triage_event(event);
    }

    #[cfg(feature = "wasm")]
    fn process_applet(&mut self) {
        let applet = match self.applet.get() {
            Some(x) => x,
            None => {
                log::info!("There are no applets. Let's process events.");
                while self.applet.get().is_none() {
                    self.wait_event();
                }
                return;
            }
        };
        let call = match applet.store_mut().last_call() {
            Some(x) => x,
            None => {
                if applet.has_handlers() {
                    // Continue processing events when main exits and at least one callback is
                    // registered.
                    let _ = self.process_event();
                } else {
                    self.stop_applet(ExitStatus::Exit);
                }
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

    #[allow(dead_code)] // in case there are no applet-controlled events
    fn disable_event(&mut self, key: Key<B>) -> Result<(), Trap> {
        if let Some(applet) = self.applet.get() {
            applet.disable(key)?;
        }
        self.flush_events();
        Ok(())
    }

    #[cfg(feature = "wasm")]
    fn call(&mut self, inst: InstId, name: &str, args: &[u32]) {
        log::debug!("Schedule thread {}{:?}.", name, args);
        let args = args.iter().map(|&x| Val::I32(x)).collect();
        #[cfg(feature = "internal-debug")]
        self.perf.record(perf::Slot::Platform);
        let applet = self.applet.get().unwrap();
        let answer = applet.store_mut().invoke(inst, name, args).map(|x| x.forget());
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
                self.applet.get().unwrap().done();
            }
            Ok(RunAnswer::Host) => (),
            Err(interpreter::Error::Trap) => applet_trapped::<B>(self, None),
            Err(e) => log::panic!("{}", log::Debug2Format(&e)),
        }
    }
}

fn applet_trapped<B: Board>(scheduler: &mut Scheduler<B>, reason: Option<&'static str>) {
    match reason {
        None => log::warn!("Applet trapped in wasm (think segfault)."),
        Some("sa") => log::warn!("Applet aborted (probably a panic)."),
        Some("se") => log::info!("Applet exited."),
        Some(name) => log::warn!("Applet trapped calling host {:?}.", name),
    }
    scheduler.stop_applet(match reason {
        Some("se") => ExitStatus::Exit,
        Some("sa") => ExitStatus::Abort,
        _ => ExitStatus::Trap,
    });
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
