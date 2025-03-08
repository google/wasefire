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

use crate::toctou::*;

// TODO(dev/fast-interp): Add debug asserts when `off` is positive and negative.
pub fn offset_front<T, M: Mode>(cur: &[T], off: isize) -> MResult<&[T], M> {
    M::choose(
        || unsafe {
            Option::from(core::slice::from_raw_parts(
                cur.as_ptr().offset(off),
                (cur.len() as isize - off) as usize,
            ))
        },
        || unsafe {
            core::slice::from_raw_parts(
                cur.as_ptr().offset(off),
                (cur.len() as isize - off) as usize,
            )
        },
    )
}
