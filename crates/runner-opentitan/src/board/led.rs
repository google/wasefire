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

use earlgrey::{GPIO, PINMUX_AON};
use wasefire_board_api::led::Api;
use wasefire_board_api::{Id, Support};
use wasefire_error::Error;

pub enum Impl {}

impl Support<usize> for Impl {
    const SUPPORT: usize = 1;
}

impl Api for Impl {
    fn get(led: Id<Self>) -> Result<bool, Error> {
        Ok((GPIO.direct_out().read_raw() & 1 << *led) != 0)
    }

    fn set(led: Id<Self>, on: bool) -> Result<(), Error> {
        let mask = 1 << *led;
        GPIO.masked_out_lower().reset().mask(mask).data(if on { mask } else { 0 }).reg.write();
        Ok(())
    }
}

pub fn init() {
    // TODO: Generate top_earlgrey_pinmux_{mio_out,outsel}. This should be MioOutIor10 and
    // GpioGpio0.
    PINMUX_AON.mio_outsel(43).reset().out(3).reg.write();
    let mask = (1 << Impl::SUPPORT) - 1;
    GPIO.masked_oe_lower().reset().mask(mask).data(mask).reg.write();
}
