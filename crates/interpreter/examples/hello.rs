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

#![allow(unused_crate_dependencies)]

use wasefire_interpreter::*;

fn main() {
    // Create an empty store.
    let mut store = Store::default();

    // Link a "println" function in module "env" in the store. This function takes 2 arguments and
    // doesn't return anything. The 2 arguments are the pointer and length of a buffer in the
    // calling module memory. This buffer is printed as a string.
    store.link_func("env", "println", 2, 0).unwrap();

    // Validate a module. Here, we have `wasm2wat hello.wasm` print:
    //
    //     (module
    //       (import "env" "println" (func $println (param i32 i32)))
    //       (func (export "main") (call $println (i32.const 0) (i32.const 5)))
    //       (memory (export "memory") 1)
    //       (data (i32.const 0) "hello"))
    //
    // The module exports its memory (just one 64kB page) and a "main" function. The memory is
    // initialized with "hello" at address zero. The "main" function simply calls into the imported
    // function "println" from module "env" with arguments 0 and 5.
    let module = Module::new(include_bytes!("hello.wasm")).unwrap();

    // Allocate memory for the module. The module needs one 64kB page, but we know it doesn't use
    // more than 5 bytes. The interpreter supports smaller memory and traps the module if it
    // accesses outside the actual memory size. This behavior is not compliant but necessary when
    // the host does neither have enough memory nor virtual memory.
    let mut memory = [0; 5];

    // Instantiate the module in the store.
    let inst = store.instantiate(module, &mut memory).unwrap();

    // Call the "main" function exported by the instance.
    let mut result = store.invoke(inst, "main", vec![]).unwrap();

    // Process calls from the module to the host until "main" terminates.
    loop {
        let mut call = match result {
            RunResult::Host(call) => call,
            RunResult::Done(results) => {
                // In our case, the exported function "main" doesn't return any results.
                assert!(results.is_empty());
                break;
            }
        };

        // We only linked one function, which has thus index zero.
        assert_eq!(call.index(), 0);

        // Get the arguments to the call. We know there are 2 of them of type i32.
        let args = call.args();
        assert_eq!(args.len(), 2);
        let ptr = args[0].unwrap_i32() as usize;
        let len = args[1].unwrap_i32() as usize;

        // Read the string from the module memory and print it.
        match call.mem_mut().get(ptr .. ptr + len) {
            Some(slice) => match std::str::from_utf8(slice) {
                Ok(string) => println!("Output: {string}"),
                Err(_) => println!("Output (not UTF-8): {slice:02x?}"),
            },
            None => {
                println!("Error: The module tried out-of-bound memory access.");
                // We drop the store to avoid accidental usage.
                drop(store);
                break;
            }
        }

        // We reply to the call with the results from "println", which has none. This in turns
        // provide the next runtime result indicating the possible next call to the host or the end
        // of execution.
        result = call.resume(&[]).unwrap();
    }
}
