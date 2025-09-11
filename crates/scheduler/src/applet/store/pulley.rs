// Copyright 2025 Google LLC
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

#![allow(const_item_mutation)]

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use wasefire_common::ptr::ExclusivePtr;
use wasefire_error::{Code, Error};
use wasefire_logger as log;
use wasefire_slice_cell::SliceCell;
use wasefire_sync::TakeCell;
use wasmtime::{
    AsContextMut, Caller, Config, Engine, ExternType, Func, Instance, Linker, Module,
    Store as WtStore, StoreContextMut, Val, ValType,
};

use super::StoreApi;
use crate::Trap;

pub struct PreStore {
    engine: Engine,
    store: WtStore<()>,
    linker: Linker<()>,
}

impl Default for PreStore {
    fn default() -> Self {
        let mut config = Config::new();
        config.async_stack_size(16 * 1024);
        config.async_support(true);
        config.max_wasm_stack(8 * 1024);
        config.memory_init_cow(false);
        config.memory_reservation(0);
        config.memory_reservation_for_growth(0);
        config.wasm_relaxed_simd(false);
        config.wasm_simd(false);
        let engine = Engine::new(&config).unwrap();
        let store = WtStore::new(&engine, ());
        let linker = Linker::new(&engine);
        PreStore { engine, store, linker }
    }
}

impl PreStore {
    pub fn link_func(&mut self, id: usize, name: &str, params: usize) -> Result<(), Error> {
        let item = match params {
            0 => Func::wrap_async(&mut self.store, move |caller, ()| call(caller, id, vec![])),
            1 => Func::wrap_async(&mut self.store, move |caller, args: (u32,)| {
                call(caller, id, vec![args.0])
            }),
            2 => Func::wrap_async(&mut self.store, move |caller, args: (u32, u32)| {
                call(caller, id, vec![args.0, args.1])
            }),
            3 => Func::wrap_async(&mut self.store, move |caller, args: (u32, u32, u32)| {
                call(caller, id, vec![args.0, args.1, args.2])
            }),
            4 => Func::wrap_async(&mut self.store, move |caller, args: (u32, u32, u32, u32)| {
                call(caller, id, vec![args.0, args.1, args.2, args.3])
            }),
            5 => {
                Func::wrap_async(&mut self.store, move |caller, args: (u32, u32, u32, u32, u32)| {
                    call(caller, id, vec![args.0, args.1, args.2, args.3, args.4])
                })
            }
            6 => Func::wrap_async(
                &mut self.store,
                move |caller, args: (u32, u32, u32, u32, u32, u32)| {
                    call(caller, id, vec![args.0, args.1, args.2, args.3, args.4, args.5])
                },
            ),
            7 => Func::wrap_async(
                &mut self.store,
                move |caller, args: (u32, u32, u32, u32, u32, u32, u32)| {
                    call(caller, id, vec![args.0, args.1, args.2, args.3, args.4, args.5, args.6])
                },
            ),
            8 => Func::wrap_async(
                &mut self.store,
                move |caller, args: (u32, u32, u32, u32, u32, u32, u32, u32)| {
                    call(
                        caller,
                        id,
                        vec![args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7],
                    )
                },
            ),
            _ => {
                log::error!("Cannot link {} taking {} parameters.", name, params);
                return Err(Error::internal(Code::NotImplemented));
            }
        };
        self.linker.define(&mut self.store, "env", name, item).map_err(convert)?;
        Ok(())
    }

    // Safety: the slice must outlive the store.
    pub unsafe fn instantiate(mut self, pulley: &'static [u8], id: usize) -> Result<Store, Error> {
        let Ok(module) = (unsafe { Module::deserialize_raw(&self.engine, pulley.into()) }) else {
            log::warn!("Failed to deserialize pulley module.");
            return Err(Error::user(Code::InvalidArgument));
        };
        'imports: for import in module.imports().filter(|x| x.module() == "env") {
            let name = import.name();
            let ExternType::Func(func) = import.ty() else { continue };
            let None = self.linker.get(&mut self.store, "env", name) else { continue };
            let mut params = 0;
            for param in func.params() {
                let ValType::I32 = param else { continue 'imports };
                params += 1;
            }
            let Some(ValType::I32) = func.result(0) else { continue };
            let None = func.result(1) else { continue };
            self.link_func(id, name, params)?;
        }
        let instance = {
            let mut instance = self.linker.instantiate_async(&mut self.store, &module);
            let instance = unsafe { Pin::new_unchecked(&mut instance) };
            match instance.poll(&mut CONTEXT) {
                Poll::Ready(Ok(x)) => x,
                Poll::Ready(Err(e)) => {
                    log::warn!("{}", log::Debug2Format(&e));
                    return Err(Error::user(Code::InvalidArgument));
                }
                Poll::Pending => unreachable!(),
            }
        };
        if instance.get_memory(&mut self.store, "memory").is_none() {
            log::warn!("Applet does not have a memory.");
            return Err(Error::user(Code::NotFound));
        }
        Ok(Store {
            instance,
            store: Box::into_raw(Box::new(self.store)),
            threads: Vec::new(),
            calls: Vec::new(),
        })
    }
}

pub struct Store {
    instance: Instance,
    // Owned Box if threads is empty, otherwise the first thread exclusively owns it.
    store: *mut WtStore<()>,
    // There is either exactly as many calls as threads (the last thread is calling into a host
    // function) or exactly one less (the last thread is running). The caller of each call is
    // exclusively borrowed by the next thread. So the last caller is owned when there are as many
    // callers as threads.
    threads: Vec<Thread>,
    calls: Vec<Call>,
}

type Thread = Pin<Box<dyn Future<Output = ThreadResult>>>;
type ThreadResult = Result<Vec<Val>, Trap>;

impl Drop for Store {
    fn drop(&mut self) {
        // It is important to drop the threads and calls in an interleaved manner since that's how
        // they depend on each other.
        loop {
            if self.threads.len() == self.calls.len() {
                if self.threads.is_empty() {
                    break;
                }
                self.calls.pop();
            } else {
                assert_eq!(self.threads.len(), self.calls.len() + 1);
                self.threads.pop();
            }
        }
        drop(unsafe { Box::from_raw(self.store) });
    }
}

pub enum RunResult {
    Done(Vec<Val>),
    Host,
    Trap,
}

impl Store {
    pub fn invoke(&mut self, name: &str, args: &[u32], nres: usize) -> Result<RunResult, Error> {
        let mut context = self.context();
        let Some(func) = self.instance.get_func(&mut context, name) else {
            return Err(Error::internal(Code::NotFound));
        };
        let params: Vec<_> = args.iter().map(|&x| Val::I32(x as i32)).collect();
        self.threads.push(Box::pin(async move {
            let mut results = vec![Val::ExternRef(None); nres];
            match func.call_async(context, &params, &mut results).await {
                Ok(()) => Ok(results),
                Err(_) => Err(Trap),
            }
        }));
        Ok(self.execute())
    }

    pub fn resume(&mut self, result: u32) -> Result<RunResult, Error> {
        assert_eq!(self.calls.len(), self.threads.len());
        self.calls.pop();
        STATE.put(State::Reply(result));
        Ok(self.execute())
    }

    pub fn last_call(&self) -> Option<&Call> {
        self.calls.last()
    }

    fn execute(&mut self) -> RunResult {
        assert_eq!(self.calls.len() + 1, self.threads.len());
        let thread = self.threads.last_mut().unwrap().as_mut();
        match thread.poll(&mut CONTEXT) {
            Poll::Ready(Ok(x)) => {
                self.threads.pop();
                RunResult::Done(x)
            }
            Poll::Ready(Err(Trap)) => RunResult::Trap,
            Poll::Pending => {
                let Some(State::Call(call)) = STATE.get() else { unreachable!() };
                self.calls.push(call);
                RunResult::Host
            }
        }
    }

    fn context(&mut self) -> StoreContextMut<'static, ()> {
        assert_eq!(self.calls.len(), self.threads.len());
        if self.threads.is_empty() {
            unsafe { &mut *self.store }.as_context_mut()
        } else {
            unsafe { &mut *self.calls.last().unwrap().caller.0 }.as_context_mut()
        }
    }

    fn memory(&mut self) -> &mut [u8] {
        let mut context = self.context();
        let memory = self.instance.get_memory(&mut context, "memory").unwrap();
        memory.data_mut(context)
    }
}

pub struct Call {
    // This is an owned box. The lifetime is bound to the thread of this call (the one at the same
    // index in the store).
    caller: ExclusivePtr<Caller<'static, ()>>,
    pub id: usize,
    pub args: Vec<u32>,
}

impl Drop for Call {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.caller.0) });
    }
}

enum State {
    Call(Call),
    Reply(u32),
}

static STATE: TakeCell<State> = TakeCell::new(None);

fn call(caller: Caller<'_, ()>, id: usize, args: Vec<u32>) -> Box<dyn Future<Output = u32> + Send> {
    let mut caller =
        Some(unsafe { core::mem::transmute::<Caller<'_, ()>, Caller<'static, ()>>(caller) });
    let mut args = Some(args);
    Box::new(core::future::poll_fn(move |_| match STATE.get() {
        None => {
            STATE.put(State::Call(Call {
                caller: ExclusivePtr(Box::into_raw(Box::new(caller.take().unwrap()))),
                id,
                args: args.take().unwrap(),
            }));
            Poll::Pending
        }
        Some(State::Reply(x)) => Poll::Ready(x),
        Some(State::Call(_)) => unreachable!(),
    }))
}

impl StoreApi for Store {
    type Memory<'a>
        = Memory<'a>
    where Self: 'a;

    fn memory(&mut self) -> Memory<'_> {
        Memory::new(self)
    }
}

pub struct Memory<'a> {
    store: *mut Store,
    memory: SliceCell<'a, u8>,
}

impl<'a> Memory<'a> {
    fn new(store: &'a mut Store) -> Self {
        Self { store, memory: SliceCell::new(store.memory()) }
    }

    fn with_store<T>(&mut self, f: impl FnOnce(&mut Store) -> T) -> T {
        self.memory.reset(); // only to free the internal state early
        let store = unsafe { &mut *self.store };
        let result = f(store);
        *self = Memory::new(store);
        result
    }
}

impl wasefire_board_api::applet::Memory for Memory<'_> {
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
            // TODO: We should ideally account this to the applet performance time.
            match store.invoke("alloc", &[size, align], 1) {
                Ok(RunResult::Done(xs)) => match xs[..] {
                    [Val::I32(0)] => log::warn!("applet is out of memory"),
                    [Val::I32(x)] => return Ok(x as u32),
                    _ => log::warn!("applet returned expected type"),
                },
                Ok(RunResult::Host) => log::warn!("alloc called into host"),
                Ok(RunResult::Trap) => log::warn!("alloc trapped"),
                Err(x) => log::error!("alloc failed with {}", x),
            }
            Err(Trap)
        })
    }
}

const CONTEXT: Context<'static> = Context::from_waker(Waker::noop());

fn convert<T>(_: T) -> Error {
    Error::new(0x8a, 0)
}

// Those symbols are needed by Wasmtime.
// TODO(wasmtime > 36.0.2): Remove everything except the thread symbols at the end.
#[allow(unused_variables)] // TODO: Remove too.
mod capi {
    #[expect(non_camel_case_types)]
    enum wasmtime_memory_image {}

    #[unsafe(no_mangle)]
    extern "C" fn wasmtime_mmap_new(size: usize, prot_flags: u32, ret: &mut *mut u8) -> i32 {
        unimplemented!();
    }
    #[unsafe(no_mangle)]
    extern "C" fn wasmtime_mmap_remap(addr: *mut u8, size: usize, prot_flags: u32) -> i32 {
        unimplemented!();
    }
    #[unsafe(no_mangle)]
    extern "C" fn wasmtime_munmap(ptr: *mut u8, size: usize) -> i32 {
        unimplemented!();
    }
    #[unsafe(no_mangle)]
    extern "C" fn wasmtime_mprotect(ptr: *mut u8, size: usize, prot_flags: u32) -> i32 {
        unimplemented!();
    }
    #[unsafe(no_mangle)]
    extern "C" fn wasmtime_page_size() -> usize {
        unimplemented!();
    }

    #[unsafe(no_mangle)]
    extern "C" fn wasmtime_memory_image_new(
        ptr: *const u8, len: usize, ret: &mut *mut wasmtime_memory_image,
    ) -> i32 {
        unimplemented!();
    }
    #[unsafe(no_mangle)]
    extern "C" fn wasmtime_memory_image_map_at(
        image: *mut wasmtime_memory_image, addr: *mut u8, len: usize,
    ) -> i32 {
        unimplemented!();
    }
    #[unsafe(no_mangle)]
    extern "C" fn wasmtime_memory_image_free(image: *mut wasmtime_memory_image) {
        unimplemented!();
    }

    static mut TLS_PTR: *mut u8 = core::ptr::null_mut();
    #[unsafe(no_mangle)]
    extern "C" fn wasmtime_tls_get() -> *mut u8 {
        unsafe { TLS_PTR }
    }
    #[unsafe(no_mangle)]
    extern "C" fn wasmtime_tls_set(ptr: *mut u8) {
        unsafe { TLS_PTR = ptr }
    }
}
