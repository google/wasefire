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
    let target = if std::env::var_os("CARGO_FEATURE_TARGET_LINUX").is_some() {
        Target::Linux
    } else if std::env::var_os("CARGO_FEATURE_TARGET_NORDIC").is_some() {
        Target::Nordic
    } else if std::env::var_os("CARGO_FEATURE_TARGET_RISCV").is_some() {
        Target::Riscv
    } else {
        panic!("one of target-{{linux,nordic,riscv}} must be enabled")
    };
    let runtime = if std::env::var_os("CARGO_FEATURE_RUNTIME_BASE").is_some() {
        Runtime::Base
    } else if std::env::var_os("CARGO_FEATURE_RUNTIME_WASM3").is_some() {
        Runtime::Wasm3
    } else if std::env::var_os("CARGO_FEATURE_RUNTIME_WASMI").is_some() {
        Runtime::Wasmi
    } else if std::env::var_os("CARGO_FEATURE_RUNTIME_WASMTIME").is_some() {
        Runtime::Wasmtime
    } else {
        panic!("one of runtime-{{base,wasm3,wasmi,wasmtime}} must be enabled")
    };
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let memory = match target {
        Target::Linux => None,
        Target::Nordic => Some("memory-nordic.x"),
        Target::Riscv => Some("memory-riscv.x"),
    };
    if let Some(memory) = memory {
        println!("cargo::rerun-if-changed={memory}");
        std::fs::copy(memory, out.join("memory.x")).unwrap();
        println!("cargo::rustc-link-search={}", out.display());
    }
    const PATH: &str = "../../third_party/wasm3/wasm-coremark/coremark-minimal.wasm";
    println!("cargo::rerun-if-changed={PATH}");
    let mut module = std::fs::read(PATH).unwrap();
    if runtime == Runtime::Wasmtime && target.is_embedded() {
        let mut config = wasmtime::Config::new();
        config.target("pulley32").unwrap();
        let engine = wasmtime::Engine::new(&config).unwrap();
        module = engine.precompile_module(&module).unwrap();
    }
    println!("cargo::warning=module size is {} bytes", module.len());
    std::fs::write(out.join("module.bin"), &module).unwrap();
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Target {
    Linux,
    Nordic,
    Riscv,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Runtime {
    Base,
    Wasm3,
    Wasmi,
    Wasmtime,
}

impl Target {
    fn is_embedded(self) -> bool {
        match self {
            Target::Linux => false,
            Target::Nordic => true,
            Target::Riscv => true,
        }
    }
}
