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

#![cfg_attr(not(any(test, feature = "log")), no_std)]

#[cfg(not(feature = "defmt"))]
mod no_defmt {
    use core::fmt::{Debug, Display, Formatter, Result};
    pub use core::panic;

    pub struct Debug2Format<T>(pub T);
    impl<T: Debug> Display for Debug2Format<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            self.0.fmt(f)
        }
    }

    pub struct Display2Format<T>(pub T);
    impl<T: Display> Display for Display2Format<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            self.0.fmt(f)
        }
    }
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
mod custom {
    #[macro_export]
    macro_rules! println {
        ($($args: expr),*$(,)?) => { if false { $(let _ = $args;)* } };
    }

    #[macro_export]
    macro_rules! trace {
        ($($args: expr),*$(,)?) => { if false { $(let _ = $args;)* } };
    }

    #[macro_export]
    macro_rules! debug {
        ($($args: expr),*$(,)?) => { if false { $(let _ = $args;)* } };
    }

    #[macro_export]
    macro_rules! info {
        ($($args: expr),*$(,)?) => { if false { $(let _ = $args;)* } };
    }

    #[macro_export]
    macro_rules! warn {
        ($($args: expr),*$(,)?) => { if false { $(let _ = $args;)* } };
    }

    #[macro_export]
    macro_rules! error {
        ($($args: expr),*$(,)?) => { if false { $(let _ = $args;)* } };
    }
}

#[cfg(feature = "log")]
pub use std::println;

#[cfg(feature = "defmt")]
pub use defmt::{
    Debug2Format, Display2Format, debug, error, flush, info, panic, println, trace, warn,
};
#[cfg(feature = "log")]
pub use log::{debug, error, info, trace, warn};
#[cfg(not(feature = "defmt"))]
pub use no_defmt::*;

#[cfg(feature = "log")]
pub fn flush() {
    log::logger().flush();
}
#[cfg(not(any(feature = "log", feature = "defmt")))]
pub fn flush() {}

#[cfg(not(feature = "defmt"))]
pub trait MaybeFormat {}
#[cfg(not(feature = "defmt"))]
impl<T> MaybeFormat for T {}

#[cfg(feature = "defmt")]
pub trait MaybeFormat: defmt::Format {}
#[cfg(feature = "defmt")]
impl<T: defmt::Format> MaybeFormat for T {}
