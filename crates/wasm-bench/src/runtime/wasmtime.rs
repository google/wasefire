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

use alloc::boxed::Box;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use wasefire_sync::TakeCell;
use wasmtime::*;

pub(crate) fn run(wasm: &[u8]) -> f32 {
    let mut config = Config::new();
    config.async_support(true);
    config.async_stack_size(16 * 1024);
    config.max_wasm_stack(8 * 1024);
    config.memory_reservation_for_growth(0);
    config.memory_init_cow(false);
    config.memory_reservation(0);
    config.wasm_relaxed_simd(false);
    config.wasm_simd(false);
    let engine = Engine::new(&config).unwrap();
    let module = unsafe { Module::deserialize_raw(&engine, wasm.into()) }.unwrap();
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);
    static CLOCK: TakeCell<u64> = TakeCell::new(None);
    let clock_ms = Func::wrap_async(
        &mut store,
        |_: Caller<'_, ()>, _: ()| -> Box<dyn Future<Output = u64> + Send> {
            Box::new(core::future::poll_fn(|_| match CLOCK.get() {
                Some(x) => Poll::Ready(x),
                None => Poll::Pending,
            }))
        },
    );
    linker.define(&mut store, "env", "clock_ms", clock_ms).unwrap();
    let mut context = Context::from_waker(Waker::noop());
    let instance = {
        let mut instance = linker.instantiate_async(&mut store, &module);
        let instance = unsafe { Pin::new_unchecked(&mut instance) };
        match instance.poll(&mut context) {
            Poll::Ready(x) => x.unwrap(),
            Poll::Pending => unreachable!(),
        }
    };
    let func = instance.get_typed_func::<(), f32>(&mut store, "run").unwrap();
    let mut thread = func.call_async(&mut store, ());
    let mut thread = unsafe { Pin::new_unchecked(&mut thread) };
    loop {
        match thread.as_mut().poll(&mut context) {
            core::task::Poll::Ready(x) => break x.unwrap(),
            core::task::Poll::Pending => {
                assert_eq!(CLOCK.replace(crate::target::clock_ms()), None)
            }
        }
    }
}

#[cfg(feature = "_target-embedded")]
mod emdedded {
    #[unsafe(no_mangle)]
    extern "C" fn wasmtime_tls_get() -> *mut u8 {
        unsafe { TLS_PTR }
    }

    #[unsafe(no_mangle)]
    extern "C" fn wasmtime_tls_set(ptr: *mut u8) {
        unsafe { TLS_PTR = ptr }
    }

    static mut TLS_PTR: *mut u8 = core::ptr::null_mut();
}
