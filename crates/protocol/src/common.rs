// Copyright 2026 Google LLC
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

use alloc::borrow::{Cow, ToOwned};
use core::cmp::Ordering;
use core::ops::{Deref, DerefMut};

use wasefire_error::{Code, Error};
use wasefire_wire::Wire;

/// String made of ASCII graphic characters only.
#[derive(Debug, Default, Clone, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[wire(refine = Name::new)]
pub struct Name<'a>(Cow<'a, str>);

impl<'a> Deref for Name<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "host")]
impl<'a> core::str::FromStr for Name<'a> {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some(bad) = input.chars().find(|x| !x.is_ascii_graphic()) {
            anyhow::bail!("{input:?} contains {bad:?} which is not a graphic ASCII character");
        }
        Ok(Name(alloc::string::String::from(input).into()))
    }
}

#[cfg(feature = "host")]
impl<'a> core::fmt::Display for Name<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> Name<'a> {
    pub fn new(data: Cow<'a, str>) -> Result<Self, Error> {
        if data.bytes().all(|x| x.is_ascii_graphic()) {
            Ok(Self(data))
        } else {
            Err(Error::user(Code::InvalidArgument))
        }
    }

    pub fn static_clone(&self) -> Name<'static> {
        Name(Cow::Owned(<str as ToOwned>::to_owned(self)))
    }

    pub fn reborrow(&self) -> Name<'_> {
        Name(<&str>::into(self))
    }
}

/// Natural number represented in big-endian (possibly with leading zeroes).
///
/// This is parsed from and pretty-printed to hexadecimal (including leading zeroes). Leading zeroes
/// are ignored during comparison (the represented natural numbers are compared).
#[derive(Debug, Default, Clone, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hexa<'a>(pub Cow<'a, [u8]>);

impl<'a> Deref for Hexa<'a> {
    type Target = Cow<'a, [u8]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for Hexa<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "host")]
impl<'a> core::str::FromStr for Hexa<'a> {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match data_encoding::HEXLOWER_PERMISSIVE.decode(input.as_bytes()) {
            Ok(x) => Ok(Hexa(x.into())),
            Err(e) => anyhow::bail!("{input:?} is not hexadecimal: {e}"),
        }
    }
}

#[cfg(feature = "host")]
impl<'a> core::fmt::Display for Hexa<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        data_encoding::HEXLOWER.encode_display(&self.0).fmt(f)
    }
}

impl<'a> PartialOrd for Hexa<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Hexa<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        let left = self.shrink();
        let right = other.shrink();
        left.len().cmp(&right.len()).then_with(|| left.cmp(right))
    }
}

impl<'a> PartialEq for Hexa<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<'a> Eq for Hexa<'a> {}

impl<'a> Hexa<'a> {
    pub fn reborrow(&self) -> Hexa<'_> {
        Hexa(<&[u8]>::into(self))
    }
    /// Strips the leading zeroes.
    pub fn shrink(&self) -> &[u8] {
        let mut result = &*self.0;
        while let Some((&0, x)) = result.split_first() {
            result = x;
        }
        result
    }

    /// Extends with leading zeroes.
    pub fn extend(&mut self, len: usize) {
        if len <= self.len() {
            return;
        }
        let cur = self.0.to_mut();
        let mid = cur.len();
        cur.resize(len, 0);
        cur.rotate_left(mid);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AppletKind {
    Wasm,
    Pulley,
    Native,
}

#[cfg(feature = "host")]
impl AppletKind {
    pub fn name(&self) -> &'static str {
        match self {
            AppletKind::Wasm => "wasm",
            AppletKind::Pulley => "pulley",
            AppletKind::Native => "native",
        }
    }
}

#[cfg(feature = "host")]
impl core::fmt::Display for AppletKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_cmp() {
        #[track_caller]
        fn test(left: &[u8], right: &[u8], expected: Ordering) {
            let actual = Hexa(left.into()).cmp(&Hexa(right.into()));
            assert_eq!(actual, expected);
        }
        test(&[], &[], Ordering::Equal);
        test(&[0], &[], Ordering::Equal);
        test(&[], &[0], Ordering::Equal);
        test(&[0], &[0], Ordering::Equal);
        test(&[], &[1], Ordering::Less);
        test(&[1], &[], Ordering::Greater);
        test(&[1], &[0, 0], Ordering::Greater);
        test(&[1, 2], &[2, 1], Ordering::Less);
    }

    #[test]
    fn version_extend() {
        #[track_caller]
        fn test(input: &[u8], len: usize, expected: &[u8]) {
            let mut actual = Hexa(input.into());
            actual.extend(len);
            assert_eq!(&**actual, expected);
            let mut actual = Hexa(input.to_vec().into());
            actual.extend(len);
            assert_eq!(&**actual, expected);
        }
        test(&[], 0, &[]);
        test(&[], 2, &[0, 0]);
        test(&[1, 1], 0, &[1, 1]);
        test(&[1, 2], 6, &[0, 0, 0, 0, 1, 2]);
    }
}
