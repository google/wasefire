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

use core::ops::Not;

impl Not for crate::led::Status {
    type Output = Self;
    fn not(self) -> Self {
        use crate::led::Status;
        match self {
            Status::Off => Status::On,
            Status::On => Status::Off,
        }
    }
}

impl crate::crypto::hash::Algorithm {
    /// Returns the length in bytes of the algorithm digest.
    pub const fn digest_len(self) -> usize {
        match self {
            crate::crypto::hash::Algorithm::Sha256 => 32,
            crate::crypto::hash::Algorithm::Sha384 => 48,
        }
    }
}

impl crate::crypto::ec::Curve {
    /// Returns the length in bytes of an integer.
    pub const fn int_len(self) -> usize {
        match self {
            crate::crypto::ec::Curve::P256 => 32,
            crate::crypto::ec::Curve::P384 => 48,
        }
    }
}

impl core::ops::Sub for crate::debug::Perf {
    type Output = crate::debug::Perf;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            platform: self.platform - rhs.platform,
            applets: self.applets - rhs.applets,
            waiting: self.waiting - rhs.waiting,
        }
    }
}

impl core::ops::SubAssign for crate::debug::Perf {
    fn sub_assign(&mut self, rhs: Self) {
        self.platform -= rhs.platform;
        self.applets -= rhs.applets;
        self.waiting -= rhs.waiting;
    }
}
