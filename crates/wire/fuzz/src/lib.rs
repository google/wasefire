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

use wasefire_error::{Code, Error};
use wasefire_wire::schema::View;
use wasefire_wire::{decode_prefix, encode, Wire};

pub fn correct(mut data: &[u8], tag_: &mut Option<u8>) -> Result<usize, Error> {
    let initial_data = data;
    let tag = decode_prefix::<u8>(&mut data)?;
    *tag_ = Some(tag);
    let value = match tag {
        0 => check::<bool>(&mut data)?,
        1 => check::<u8>(&mut data)?,
        2 => check::<i8>(&mut data)?,
        3 => check::<u16>(&mut data)?,
        4 => check::<i16>(&mut data)?,
        5 => check::<u32>(&mut data)?,
        6 => check::<i32>(&mut data)?,
        7 => check::<u64>(&mut data)?,
        8 => check::<i64>(&mut data)?,
        9 => check::<usize>(&mut data)?,
        10 => check::<isize>(&mut data)?,
        11 => check::<&str>(&mut data)?,
        50 => check::<&[u8; 0]>(&mut data)?,
        51 => check::<&[u8; 1]>(&mut data)?,
        52 => check::<&[u8; 2]>(&mut data)?,
        53 => check::<&[u8]>(&mut data)?,
        54 => check::<[&str; 0]>(&mut data)?,
        55 => check::<[&str; 1]>(&mut data)?,
        56 => check::<[&str; 2]>(&mut data)?,
        57 => check::<Vec<&str>>(&mut data)?,
        58 => check::<Option<&str>>(&mut data)?,
        59 => check::<Result<&str, &str>>(&mut data)?,
        _ => return Err(Error::world(Code::OutOfBounds)),
    };
    let len = initial_data.len() - data.len() - 1;
    assert_eq!(initial_data[1 ..][.. len], value[..]);
    Ok(len)
}

fn check<'a, T: Wire<'a>>(data: &mut &'a [u8]) -> Result<Box<[u8]>, Error> {
    let data = encode(&decode_prefix::<T>(data)?)?;
    assert_eq!(View::new::<T>().validate(&data), Ok(()));
    Ok(data)
}
