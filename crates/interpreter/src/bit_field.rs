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

pub fn into_signed_field(mask: u32, value: i32) -> Result<u32, Error> {
    into_field(mask, value.wrapping_add(offset(mask)) as u32)
}

pub fn from_signed_field(mask: u32, field: u32) -> i32 {
    from_field(mask, field) as i32 - offset(mask)
}

fn offset(mask: u32) -> i32 {
    1 << (mask.count_ones() - 1)
}

pub fn into_field(mask: u32, value: u32) -> Result<u32, Error> {
    let field = (value << mask.trailing_zeros()) & mask;
    if from_field(mask, field) != value {
        return Err(unsupported(if_debug!(Unsupported::SideTable)));
    }
    Ok(field)
}

pub fn from_field(mask: u32, field: u32) -> u32 {
    (field & mask) >> mask.trailing_zeros()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn signed_field_round_trip() {
        let mask = 0b1111000;
        for expected in -8i32 .. 8 {
            let actual = from_signed_field(mask, into_signed_field(mask, expected).unwrap());
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn signed_field_error_positive() {
        for expected in 8 .. 100 {
            assert!(matches!(into_signed_field(0b1111000, expected), Err(Error::Unsupported(_))));
        }
    }

    #[test]
    fn signed_field_error_negative() {
        for expected in -100 .. -8 {
            assert!(matches!(into_signed_field(0b1111000, expected), Err(Error::Unsupported(_))));
        }
    }

    #[test]
    fn unsigned_field_round_trip() {
        let mask = 0b111000;
        for expected in 0 .. 8 {
            let actual = from_field(mask, into_field(mask, expected).unwrap());
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn unsigned_field_error() {
        for expected in 8 .. 10 {
            assert!(matches!(into_field(0b111000, expected), Err(Error::Unsupported(_))));
        }
    }
}
