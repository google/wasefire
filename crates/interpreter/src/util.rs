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

use crate::error::*;

// TODO(dev/fast-interp): Add debug asserts when `off` is positive and negative, and `toctou`
// support.
pub fn offset_front<T>(cur: &[T], off: isize) -> &[T] {
    unsafe {
        core::slice::from_raw_parts(cur.as_ptr().offset(off), (cur.len() as isize - off) as usize)
    }
}

#[allow(dead_code)]
pub fn offset_front_check<'a, T>(beg: &'a [T], cur: &'a [T], off: isize) -> Result<&'a [T], Error> {
    let range = beg.subslice_range(cur).ok_or_else(invalid)?;
    check(range.end == beg.len())?;
    let off = range.start.checked_add_signed(off).ok_or_else(invalid)?;
    beg.get(off ..).ok_or_else(invalid)
}

#[allow(dead_code)]
#[cfg(feature = "toctou")]
fn _offset_front<'a, T>(beg: &'a [T], cur: &'a [T], off: isize) -> &'a [T] {
    let mut range = beg.subslice_range(cur).unwrap();
    range.start = range.start.strict_add_signed(off);
    &beg[range]
}

#[allow(dead_code)]
#[cfg(not(feature = "toctou"))]
fn _offset_front<T>(cur: &[T], off: isize) -> &[T] {
    let old_ptr = cur.as_ptr();
    let new_len = (cur.len() as isize - off) as usize;
    // SAFETY: There might be a provenance problem, but otherwise `cur` is derived from a larger
    // `beg` slice and `off` does not reach outside `beg`.
    unsafe { core::slice::from_raw_parts(old_ptr.offset(off), new_len) }
}
