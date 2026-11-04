// Copyright 2026 Google LLC
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

#![allow(unused_crate_dependencies)]

use wasefire_interpreter::{Error, Module, RunResult, Store, Val, prepare};
use wast::{Wat, parser};

#[test]
fn reject_invalid_host_results_without_losing_call() {
    let wat = r#"(module
        (import "env" "value" (func $value (result i32)))
        (func (export "run") (result i32)
            (call $value)
            (i32.const 1)
            (i32.add)))"#;
    let buffer = parser::ParseBuffer::new(wat).unwrap();
    let mut module: Wat = parser::parse(&buffer).unwrap();
    let wasm = prepare(&module.encode().unwrap()).unwrap();
    let module = Module::new(&wasm).unwrap();
    let mut store = Store::default();
    store.link_func("env", "value", 0, 1).unwrap();
    let mut memory = [];
    let inst = store.instantiate(module, &mut memory).unwrap();
    let RunResult::Host(call) = store.invoke(inst, "run", vec![]).unwrap() else {
        panic!("expected a host call");
    };

    assert_eq!(call.resume(&[]).unwrap_err(), Error::Invalid);
    let call = store.last_call().expect("the host call should remain pending");
    assert_eq!(call.resume(&[Val::I64(41)]).unwrap_err(), Error::Invalid);
    let call = store.last_call().expect("the host call should remain pending");
    let RunResult::Done(result) = call.resume(&[Val::I32(41)]).unwrap() else {
        panic!("expected the function to return");
    };
    assert_eq!(result, vec![Val::I32(42)]);
}
