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

fn main() {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    let memory = match () {
        () if cfg!(feature = "target-nordic") => Some("memory-nordic.x"),
        () if cfg!(feature = "target-riscv") => Some("memory-riscv.x"),
        () => None,
    };
    if let Some(memory) = memory {
        println!("cargo::rerun-if-changed={memory}");
        std::fs::copy(memory, out.join("memory.x")).unwrap();
        println!("cargo::rustc-link-search={}", out.display());
    }

    const PATH: &str = "../../third_party/wasm3/wasm-coremark/coremark-minimal.wasm";
    println!("cargo::rerun-if-changed={PATH}");
    let mut module = std::fs::read(PATH).unwrap();
    #[cfg(feature = "runtime-base")]
    {
        module = wasefire_interpreter::prepare(&module).unwrap();
    }
    #[cfg(feature = "runtime-wasmtime")]
    {
        let mut config = wasmtime::Config::new();
        config.target("pulley32").unwrap();
        // TODO(https://github.com/bytecodealliance/wasmtime/issues/10286): Also strip symbol table.
        config.generate_address_map(false);
        config.memory_init_cow(false);
        config.memory_reservation(0);
        config.wasm_relaxed_simd(false);
        config.wasm_simd(false);
        let engine = wasmtime::Engine::new(&config).unwrap();
        module = engine.precompile_module(&module).unwrap();
    }
    println!("cargo::warning=module size is {} bytes", module.len());
    std::fs::write(out.join("module.bin"), &module).unwrap();
}
