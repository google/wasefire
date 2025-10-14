// Copyright 2025 Google LLC
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

#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use wasefire_wire::Wire;

#[derive(Debug, Clone, Wire)]
pub enum Request {
    CaptureStart,
    CaptureDone,
    CaptureImage, // response is raw
    EnrollStart,
    EnrollDone,
    IdentifyStart(Option<Vec<u8>>), // id
    IdentifyDone,
    Delete(Option<Vec<u8>>), // id
    List,
}

#[derive(Debug, Clone, Wire)]
pub enum Response {
    CaptureStart,
    CaptureDone(Option<usize>), // width
    CaptureImage(Vec<u8>),      // fake variant (response is raw)
    EnrollStart,
    EnrollDone(Result<Vec<u8>, (usize, Option<usize>)>), // id, (detected, remaining)
    IdentifyStart,
    IdentifyDone(Option<Option<Vec<u8>>>), // done, match/no-match
    Delete,
    List(Vec<Vec<u8>>),
}
