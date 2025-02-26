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

use wasmi::*;

pub(crate) fn run(wasm: &[u8]) -> f32 {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::<()>::new(&engine);
    let clock_ms =
        Func::wrap(&mut store, move |_: Caller<'_, ()>| -> u64 { crate::target::clock_ms() });
    linker.define("env", "clock_ms", clock_ms).unwrap();
    let instance = linker.instantiate(&mut store, &module).unwrap().start(&mut store).unwrap();
    let func = instance.get_typed_func::<(), f32>(&store, "run").unwrap();
    func.call(&mut store, ()).unwrap()
}
