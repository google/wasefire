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

use crate::error::*;
use crate::syntax::*;

pub trait Mode {
    type Error;

    fn invalid<T>() -> Result<T, Self::Error>;

    fn unsupported<T>() -> Result<T, Self::Error>;

    fn check(cond: impl FnOnce() -> bool) -> Result<(), Self::Error>;

    fn open<T>(check: impl FnOnce() -> Option<T>) -> Result<T, Self::Error>;

    fn choose<T>(
        check: impl FnOnce() -> Option<T>, use_: impl FnOnce() -> T,
    ) -> Result<T, Self::Error>;
}

#[derive(Debug, Default, Clone)]
pub struct Check;

impl Mode for Check {
    type Error = Error;

    fn invalid<T>() -> Result<T, Self::Error> {
        Err(invalid())
    }

    fn unsupported<T>() -> Result<T, Self::Error> {
        Err(unsupported())
    }

    fn check(cond: impl FnOnce() -> bool) -> Result<(), Self::Error> {
        check(cond())
    }

    fn open<T>(check: impl FnOnce() -> Option<T>) -> Result<T, Self::Error> {
        check().ok_or_else(invalid)
    }

    fn choose<T>(
        check: impl FnOnce() -> Option<T>, _: impl FnOnce() -> T,
    ) -> Result<T, Self::Error> {
        Self::open(check)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Use;

impl Mode for Use {
    type Error = !;

    fn invalid<T>() -> Result<T, Self::Error> {
        #[cfg(not(feature = "toctou"))]
        unsafe {
            core::hint::unreachable_unchecked()
        }
        #[cfg(feature = "toctou")]
        unreachable!()
    }

    fn unsupported<T>() -> Result<T, Self::Error> {
        Self::invalid()
    }

    fn check(_cond: impl FnOnce() -> bool) -> Result<(), Self::Error> {
        Ok(())
    }

    fn open<T>(check: impl FnOnce() -> Option<T>) -> Result<T, Self::Error> {
        #[cfg(not(feature = "toctou"))]
        return Ok(unsafe { check().unwrap_unchecked() });
        #[cfg(feature = "toctou")]
        Ok(check().unwrap())
    }

    fn choose<T>(
        check: impl FnOnce() -> Option<T>, use_: impl FnOnce() -> T,
    ) -> Result<T, Self::Error> {
        #[cfg(not(feature = "toctou"))]
        {
            drop(check);
            Ok(use_())
        }
        #[cfg(feature = "toctou")]
        {
            drop(use_);
            Self::open(check)
        }
    }
}

pub type MResult<T, M> = Result<T, <M as Mode>::Error>;

pub fn get(xs: &[u8], i: usize) -> u8 {
    #[cfg(not(feature = "toctou"))]
    return unsafe { *xs.get_unchecked(i) };
    #[cfg(feature = "toctou")]
    xs[i]
}

pub fn split_at(xs: &[u8], mid: usize) -> (&[u8], &[u8]) {
    #[cfg(not(feature = "toctou"))]
    return unsafe { xs.split_at_unchecked(mid) };
    #[cfg(feature = "toctou")]
    xs.split_at(mid)
}

pub fn byte_enum<M: Mode, T: Copy + TryFromByte + UnsafeFromByte>(x: u8) -> MResult<T, M> {
    M::choose(|| T::try_from_byte(x), || unsafe { T::from_byte_unchecked(x) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_check() {
        fn test<M: Mode>(eval: bool, cond: bool, result: MResult<(), M>)
        where
            M::Error: core::fmt::Debug + PartialEq,
        {
            let mut run = false;
            assert_eq!(
                M::check(|| {
                    run = true;
                    cond
                }),
                result
            );
            assert_eq!(run, eval);
        }
        test::<Check>(true, false, Err(Error::Invalid));
        test::<Check>(true, true, Ok(()));
        test::<Use>(false, false, Ok(()));
        test::<Use>(false, true, Ok(()));
    }
}
