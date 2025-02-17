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

//! WebAssembly interpreter with customizable trade-off between footprint and performance.
//!
//! The main concepts of this crate are:
//!
//! - A [`Store`] contains instantiated modules and permits execution. Note that execution within
//!   the same store must follow a stack behavior. A function "bar" may be called while a function
//!   "foo" is running: "bar" will temporarily interrupt "foo" until "bar" returns at which point
//!   "foo" would resume. This is to avoid corrupting the stack within the same instance. Once the
//!   WebAssembly threads proposal lands, this restriction may be removed.
//!
//! - A [`Module`] represents a valid module. Only valid modules may be instantiated. A module is
//!   just a byte slice holding a WebAssembly module in binary format.
//!
//! - A [`RunResult`] represents the result of a (possibly partial) execution. When invoking an
//!   exported function from an instance in a store, or when resuming the execution after a call to
//!   the host, a `RunResult` is returned providing either the next call to the host or whether
//!   execution of the most recently invoked function terminated along with its results.
//!
//! # Examples
//!
//! For a concrete "hello" example, see `examples/hello.rs` which walks through most important steps
//! in using this crate. Otherwise, here are some short excerpts:
//!
//! Creating a store can simply use the `Default` trait:
//!
//! ```
//! # use wasefire_interpreter::*;
//! let mut store = Store::default();
//! ```
//!
//! Linking a host function in a store is done with [`Store::link_func()`]:
//!
//! ```
//! # use wasefire_interpreter::*;
//! # fn doc(store: &mut Store) -> Result<(), Error> {
//! store.link_func("env", "add", 2, 1)?;
//! # Ok(())
//! # }
//! ```
//!
//! Instantiating a valid module in a store is done with [`Store::instantiate()`]:
//!
//! ```
//! # use wasefire_interpreter::*;
//! # fn doc<'a>(store: &mut Store<'a>, module: Module<'a>, memory: &'a mut [u8])
//! # -> Result<(), Error> {
//! let inst = store.instantiate(module, memory)?;
//! # Ok(())
//! # }
//! ```
//!
//! Invoking a function exported by an instance is done with [`Store::invoke()`]:
//!
//! ```
//! # use wasefire_interpreter::*;
//! # fn doc<'a>(store: &mut Store<'a>, inst: InstId) -> Result<(), Error> {
//! let mut result = store.invoke(inst, "mul", vec![Val::I32(13), Val::I32(29)])?;
//! # Ok(())
//! # }
//! ```
//!
//! Processing a call to the host is done with the [`Call`] in [`RunResult::Host`]:
//!
//! ```
//! # use wasefire_interpreter::*;
//! # fn process<'a, 'm>(call: &mut Call<'a, 'm>) -> Result<Vec<Val>, Error> { todo!() }
//! # fn doc<'a, 'm>(mut result: RunResult<'a, 'm>) -> Result<Vec<Val>, Error> {
//! loop {
//!     let mut call = match result {
//!         RunResult::Done(results) => return Ok(results),
//!         RunResult::Host(call) => call,
//!     };
//!     let results = process(&mut call)?;
//!     result = call.resume(&results)?;
//! }
//! # }
//! ```
//!
//! # Atomic support
//!
//! This crate uses atomic operations and relies on the `portable-atomic` crate to support
//! architectures without atomic operations. Depending on your architecture, you may need one of the
//! following:
//!
//! - Enable the `portable-atomic/critical-section` feature (possibly adding a direct dependency on
//!   `portable-atomic`).
//!
//! - Set the `--cfg=portable_atomic_unsafe_assume_single_core` rustc flag.
//!
//! You can find more information in the `portable-atomic`
//! [documentation](https://docs.rs/portable-atomic/latest/portable_atomic).

#![cfg_attr(not(feature = "debug"), no_std)]
#![cfg_attr(test, allow(unused_crate_dependencies))]
#![feature(concat_idents)]
#![feature(float_minimum_maximum)]
#![feature(never_type)]
#![feature(pointer_is_aligned_to)]
#![feature(try_blocks)]
#![feature(unwrap_infallible)]

extern crate alloc;

macro_rules! if_debug {
    ($unit:expr) => {{
        #[cfg(feature = "debug")]
        let unit = $unit;
        #[cfg(not(feature = "debug"))]
        let unit = ();
        unit
    }};
}

macro_rules! support_if {
    ($feature:literal[$($var:ident)*], $then:expr, $else:expr) => {{
        #[cfg(feature = $feature)]
        {
            $then
        }
        #[cfg(not(feature = $feature))]
        {
            $( let _ = $var; )*
            $else
        }
    }};
}

mod error;
mod exec;
mod id;
mod module;
mod parser;
mod side_table;
mod syntax;
mod toctou;
mod util;
mod valid;

pub use error::{Error, TRAP_CODE, Unsupported};
pub use exec::{Call, InstId, MEMORY_ALIGN, RunAnswer, RunResult, Store, StoreId, Val};
pub use module::Module;
pub use syntax::{
    FuncType, GlobalType, ImportDesc, Limits, Mut, RefType, ResultType, TableType, ValType,
};
pub use valid::prepare;
pub use valid::merge;
