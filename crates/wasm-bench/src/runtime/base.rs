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

use alloc::vec;

use wasefire_interpreter::{FuncType, Module, RunResult, Store, Val, ValType};

pub(crate) fn run(wasm: &[u8]) -> f32 {
    const MEMORY_SIZE: usize = 0x10000;
    #[repr(align(16))]
    struct Memory([u8; MEMORY_SIZE]);
    static mut MEMORY: Memory = Memory([0; MEMORY_SIZE]);

    let mut store = Store::default();
    store
        .link_func_custom(
            "env",
            "clock_ms",
            FuncType { params: ().into(), results: ValType::I64.into() },
        )
        .unwrap();
    let module = Module::new(wasm).unwrap();
    #[allow(static_mut_refs)]
    let inst = store.instantiate(module, unsafe { &mut MEMORY.0 }).unwrap();
    let mut state = store.invoke(inst, "run", vec![]).unwrap();
    let results = loop {
        let call = match state {
            RunResult::Done(results) => break results,
            RunResult::Host(call) => call,
        };
        assert_eq!(call.inst(), inst);
        assert_eq!(call.index(), 0);
        assert_eq!(call.args().len(), 0);
        let time = crate::target::clock_ms();
        state = call.resume(&[Val::I64(time)]).unwrap();
    };
    assert_eq!(results.len(), 1);
    match results[0] {
        Val::F32(x) => f32::from_bits(x),
        _ => unreachable!(),
    }
}
