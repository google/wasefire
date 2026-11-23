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

fn module(wat: &str) -> Module<'static> {
    let buffer = parser::ParseBuffer::new(wat).unwrap();
    let mut module: Wat = parser::parse(&buffer).unwrap();
    let wasm = prepare(&module.encode().unwrap()).unwrap();
    Module::new(Vec::leak(wasm)).unwrap()
}

/// A module whose start function stores the result of a host call in a global.
const PROVIDER: &str = r#"(module
    (import "env" "value" (func $value (result i32)))
    (global $g (mut i32) (i32.const 0))
    (func $init (global.set $g (call $value)))
    (func (export "get") (result i32) (global.get $g))
    (start $init))"#;

/// A module linking to the exported function of the provider.
const USER: &str = r#"(module (import "provider" "get" (func (result i32))))"#;

#[test]
fn start_function_can_call_the_host() {
    let mut store = Store::default();
    store.link_func("env", "value", 0, 1).unwrap();
    let inst = store.instantiate(module(PROVIDER), &mut []).unwrap();
    store.set_name(inst, "provider").unwrap();

    // The start function is waiting for the host.
    let call = store.last_call().expect("the start function should call the host");
    assert_eq!(call.index(), 0);
    assert_eq!(call.inst(), inst);

    // Until it returns, the instance can neither be invoked nor linked to.
    assert_eq!(store.invoke(inst, "get", vec![]).unwrap_err(), Error::Invalid);
    assert_eq!(store.instantiate(module(USER), &mut []).unwrap_err(), Error::Invalid);

    let call = store.last_call().expect("the start function should still wait for the host");
    let RunResult::Done(result) = call.resume(&[Val::I32(41)]).unwrap() else {
        panic!("expected the start function to return");
    };
    assert!(result.is_empty());
    assert!(store.last_call().is_none());

    // Once it returned, both work.
    let RunResult::Done(result) = store.invoke(inst, "get", vec![]).unwrap() else {
        panic!("expected the function to return");
    };
    assert_eq!(result, vec![Val::I32(41)]);
    store.instantiate(module(USER), &mut []).unwrap();
}

#[test]
fn start_function_trapping_after_host_call_leaves_instance_unusable() {
    let mut store = Store::default();
    store.link_func("env", "value", 0, 1).unwrap();
    let wat = r#"(module
        (import "env" "value" (func $value (result i32)))
        (func $init (drop (call $value)) (unreachable))
        (func (export "get") (result i32) (i32.const 0))
        (start $init))"#;
    let inst = store.instantiate(module(wat), &mut []).unwrap();
    let call = store.last_call().unwrap();
    assert_eq!(call.resume(&[Val::I32(0)]).unwrap_err(), Error::Trap);
    // Instantiation failed, so the instance never becomes usable.
    assert_eq!(store.invoke(inst, "get", vec![]).unwrap_err(), Error::Invalid);
}
