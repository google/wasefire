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

use wasm3::{Environment, Module};

pub(crate) fn run(wasm: &[u8]) -> f32 {
    let env = Environment::new().unwrap();
    let rt = env.create_runtime(4096).unwrap();
    let module = Module::parse(&env, wasm).unwrap();
    let mut module = rt.load_module(module).unwrap();
    module.link_closure("env", "clock_ms", |_, ()| Ok(crate::target::clock_ms())).unwrap();
    let func = module.find_function::<(), f32>("run").unwrap();
    func.call().unwrap()
}
