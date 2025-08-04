// Copyright 2023 Google LLC
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

use wasefire_board_api::AppletMemory;

pub use self::impl_::Store;

#[cfg_attr(feature = "native", path = "store/native.rs")]
#[cfg_attr(feature = "wasm", path = "store/wasm.rs")]
mod impl_;

pub type Memory<'a> = <Store as StoreApi>::Memory<'a>;

pub trait StoreApi {
    type Memory<'a>: AppletMemory
    where Self: 'a;

    fn memory(&mut self) -> Self::Memory<'_>;
}
