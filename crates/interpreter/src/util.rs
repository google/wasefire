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
    if beg.is_empty() || cur.is_empty() {
        #[cfg(feature = "debug")]
        eprintln!("One or both slices are empty.");
        return Err(invalid());
    }
    if beg.last().unwrap() as *const T != cur.last().unwrap() as *const T {
        #[cfg(feature = "debug")]
        eprintln!("The last elements of beg and cur do not match.");
        return Err(invalid());
    }
    let mut cur_offset = beg.len().checked_sub(cur.len()).ok_or_else(|| {
        #[cfg(feature = "debug")]
        eprintln!("Cur length {} exceeds beg length {}.", cur.len(), beg.len());
        invalid()
    })?;
    if off > 0 {
        cur_offset = cur_offset.checked_add(off as usize).ok_or_else(|| {
            #[cfg(feature = "debug")]
            eprintln!("Addition overflow");
            invalid()
        })?;
        if cur_offset < cur.len() {
            #[cfg(feature = "debug")]
            eprintln!("The offset overflows the slice length.");
            return Err(invalid());
        }
    } else {
        cur_offset = cur_offset.checked_sub(off as usize).ok_or_else(|| {
            #[cfg(feature = "debug")]
            eprintln!("Subtraction overflow");
            invalid()
        })?;
        if cur_offset >= cur.len() {
            #[cfg(feature = "debug")]
            eprintln!("The offset overflows below 0.");
            return Err(invalid());
        }
    }
    Ok(&beg[cur_offset ..])
}

#[allow(dead_code)]
#[allow(unused_variables)]
pub fn offset_front_use<'a, T>(beg: &'a [T], cur: &'a [T], off: isize) -> &'a [T] {
    #[cfg(not(feature = "toctou"))]
    return offset_front_unsafe(beg, off);
    #[cfg(feature = "toctou")]
    offset_front_toctou(beg, cur, off)
}

#[allow(dead_code)]
fn offset_front_toctou<'a, T>(beg: &'a [T], cur: &'a [T], off: isize) -> &'a [T] {
    if beg.is_empty() || cur.is_empty() {
        #[cfg(feature = "debug")]
        eprintln!("One or both slices are empty.");
        panic!();
    }
    if beg.last().unwrap() as *const T != cur.last().unwrap() as *const T {
        #[cfg(feature = "debug")]
        eprintln!("The last elements of beg and cur do not match.");
        panic!();
    }
    let mut cur_offset = beg
        .len()
        .checked_sub(cur.len())
        .ok_or_else(|| {
            #[cfg(feature = "debug")]
            eprintln!("Cur length {} exceeds beg length {}.", cur.len(), beg.len());
            invalid()
        })
        .unwrap();
    if off > 0 {
        cur_offset = cur_offset
            .checked_add(off as usize)
            .ok_or_else(|| {
                #[cfg(feature = "debug")]
                eprintln!("Addition overflow");
                invalid()
            })
            .unwrap();
        if cur_offset < cur.len() {
            #[cfg(feature = "debug")]
            eprintln!("The offset overflows the slice length.");
            panic!();
        }
    } else {
        cur_offset = cur_offset
            .checked_sub(off as usize)
            .ok_or_else(|| {
                #[cfg(feature = "debug")]
                eprintln!("Subtraction overflow");
                invalid()
            })
            .unwrap();
        if cur_offset >= cur.len() {
            #[cfg(feature = "debug")]
            eprintln!("The offset overflows below 0.");
            panic!();
        }
    }
    &beg[cur_offset ..]
}

#[allow(dead_code)]
fn offset_front_unsafe<T>(cur: &[T], off: isize) -> &[T] {
    unsafe {
        core::slice::from_raw_parts(cur.as_ptr().offset(off), (cur.len() as isize - off) as usize)
    }
}
