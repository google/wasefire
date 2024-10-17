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

//! Provides macros to check mutually exclusive features.

#![no_std]

// TODO(https://github.com/rust-lang/rust/issues/122105): Remove when fixed.
extern crate alloc;

/// Makes sure exactly one of the provided feature is enabled.
///
/// For example, `exactly_one_of!["a", "b", "c"]` will expand to:
///
/// ```ignore
/// #[cfg(any(
///     not(any(feature = "a", feature = "b", feature = "c")),
///     all(feature = "a", feature = "b"),
///     all(feature = "a", feature = "c"),
///     all(feature = "b", feature = "c"),
/// ))]
/// compile_error!("Exactly one of feature \"a\", \"b\", or \"c\" can be enabled.");
/// ```
#[macro_export]
macro_rules! exactly_one_of {
    ($f:literal, $($fs:literal),+$(,)?) => {
        $crate::internal_one_of!(0 [$f $($fs)*] "Exactly");
    };
}

/// Makes sure at most one of the provided feature is enabled.
///
/// For example, `at_most_one_of!["a", "b", "c"]` will expand to:
///
/// ```ignore
/// #[cfg(any(
///     all(feature = "a", feature = "b"),
///     all(feature = "a", feature = "c"),
///     all(feature = "b", feature = "c"),
/// ))]
/// compile_error!("At most one of feature \"a\", \"b\", or \"c\" can be enabled.");
/// ```
#[macro_export]
macro_rules! at_most_one_of {
    ($f:literal, $($fs:literal),+$(,)?) => {
        $crate::internal_one_of!(1 [$f $($fs)*] () ["At most";]);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! internal_one_of {
    (0 [$($f:literal)*] $p:literal) => {
        $crate::internal_one_of!(1 [$($f)*] (not(any($(feature = $f),*)),) [$p;]);
    };

    (1 [$f:literal $($fs:literal)*] $r:tt [$($ds:literal)*; $($d:literal)?]) => {
        $crate::internal_one_of!(2 [$($fs)*] $f [$($fs)*] $r [$($ds)* $($d)?; $f]);
    };
    (1 [] $r:tt $ds:tt) => {
        #[cfg(any $r)]
        $crate::internal_one_of!(3 $ds);
    };

    (2 $s:tt $g:literal [$f:literal $($fs:literal)*] ($($r:tt)*) $d:tt) => {
        $crate::internal_one_of!(2 $s $g [$($fs)*] ($($r)* all(feature = $g, feature = $f),) $d);
    };
    (2 $s:tt $g:literal [] $r:tt $d:tt) => {
        $crate::internal_one_of!(1 $s $r $d);
    };

    (3 [$p:literal $a:literal; $b:literal]) => {
        compile_error!(concat!($p, " one of feature ", stringify!($a), " or ", stringify!($b),
          " can be enabled."));
    };
    (3 [$p:literal $($as:literal)*; $b:literal]) => {
        compile_error!(concat!($p, " one of feature ",$(stringify!($as), ", ",)* "or ",
          stringify!($b), " can be enabled."));
    };
}
