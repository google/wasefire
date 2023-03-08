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
        ($($args: expr),*) => { if false { $(let _ = $args;)* } };
    }

    #[macro_export]
    macro_rules! trace {
        ($($args: expr),*) => { if false { $(let _ = $args;)* } };
    }

    #[macro_export]
    macro_rules! debug {
        ($($args: expr),*) => { if false { $(let _ = $args;)* } };
    }

    #[macro_export]
    macro_rules! info {
        ($($args: expr),*) => { if false { $(let _ = $args;)* } };
    }

    #[macro_export]
    macro_rules! warn {
        ($($args: expr),*) => { if false { $(let _ = $args;)* } };
    }

    #[macro_export]
    macro_rules! error {
        ($($args: expr),*) => { if false { $(let _ = $args;)* } };
    }
}

#[cfg(feature = "log")]
mod custom {
    use std::time::Instant;

    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref ORIGIN: Instant = Instant::now();
    }

    #[macro_export]
    macro_rules! println {
        ($($arg:tt)*) => {
            std::print!("{:09.3}: ", $crate::ORIGIN.elapsed().as_secs_f32());
            std::println!($($arg)*);
        };
    }
}

#[cfg(not(feature = "defmt"))]
pub use custom::*;
#[cfg(feature = "defmt")]
pub use defmt::{debug, error, info, panic, println, trace, warn, Debug2Format, Display2Format};
#[cfg(feature = "log")]
pub use log::{debug, error, info, trace, warn};
#[cfg(not(feature = "defmt"))]
pub use no_defmt::*;

#[macro_export]
macro_rules! every {
    ($n:expr, $t:ident, $($args: expr),*$(,)?) => {{
        use core::sync::atomic::AtomicU32;
        use core::sync::atomic::Ordering::SeqCst;
        static COUNT: AtomicU32 = AtomicU32::new(0);
        let count = COUNT.fetch_add(1, SeqCst).wrapping_add(1);
        if count % $n == 0 {
            $crate::$t!($($args),*);
        }
    }};
}
