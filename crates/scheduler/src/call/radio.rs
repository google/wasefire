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

use wasefire_applet_api::radio::Api;
use wasefire_board_api::Api as Board;

use crate::DispatchSchedulerCall;

#[cfg(feature = "applet-api-radio-ble")]
mod ble;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        #[cfg(feature = "applet-api-radio-ble")]
        Api::Ble(call) => ble::process(call),
    }
}
