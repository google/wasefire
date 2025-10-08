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

/// Errors returned by functions in this crate.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    /// An argument is invalid.
    Invalid,

    /// An argument is not found.
    NotFound,

    /// An argument is not supported.
    Unsupported(Unsupported),

    /// Execution trapped.
    // TODO: In debug mode, trap could describe the reason (like resource exhaustion, etc).
    Trap,
}

pub const TRAP_CODE: u16 = 0xffff;

impl From<Error> for wasefire_error::Error {
    fn from(value: Error) -> Self {
        use wasefire_error::{Code, Error as WError};
        match value {
            Error::Invalid => WError::user(Code::InvalidArgument),
            Error::NotFound => WError::user(Code::NotFound),
            Error::Unsupported(_) => WError::internal(Code::NotImplemented),
            Error::Trap => WError::user(TRAP_CODE),
        }
    }
}

#[cfg(not(feature = "debug"))]
pub type Unsupported = ();

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[cfg(feature = "debug")]
pub enum Unsupported {
    Exception,
    HeapType,
    MaxLocals,
    Opcode(u8),
    OpcodeFc(u32),
    SideTable,
    TableInit,
    TailCall,
}

#[cfg(feature = "debug")]
pub fn print_backtrace() {
    let backtrace = std::backtrace::Backtrace::capture();
    if matches!(backtrace.status(), std::backtrace::BacktraceStatus::Captured) {
        println!("{backtrace}");
    }
}

pub fn invalid() -> Error {
    #[cfg(feature = "debug")]
    print_backtrace();
    Error::Invalid
}

pub fn not_found() -> Error {
    #[cfg(feature = "debug")]
    print_backtrace();
    Error::NotFound
}

pub fn unsupported(reason: Unsupported) -> Error {
    #[cfg(feature = "debug")]
    print_backtrace();
    Error::Unsupported(reason)
}

pub fn trap() -> Error {
    #[cfg(feature = "debug")]
    print_backtrace();
    Error::Trap
}

pub fn check(cond: bool) -> Result<(), Error> {
    if cond { Ok(()) } else { Err(invalid()) }
}
