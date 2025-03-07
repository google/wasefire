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

use earlgrey::RV_TIMER;

pub fn init() {
    // We assume a 24MHz clock. We want a 1MHz counter. This gives a prescale of 23.
    RV_TIMER.cfg(0).reset().prescale(23).reg.write();
    RV_TIMER.ctrl().reset().active(0, true).reg.write();
    // We always enable interrupts. We use the maximum deadline to disable.
    RV_TIMER.intr_enable(0).reset().ie(true).reg.write();
}

pub fn uptime_us() -> u64 {
    let mut high = RV_TIMER.timer_v_upper(0).read_raw();
    let mut low = RV_TIMER.timer_v_lower(0).read_raw();
    loop {
        let new_high = RV_TIMER.timer_v_upper(0).read_raw();
        if new_high == high {
            break ((high as u64) << 32) | low as u64;
        }
        high = new_high;
        low = RV_TIMER.timer_v_lower(0).read_raw();
    }
}

pub fn deadline_us(deadline: u64) {
    RV_TIMER.compare_lower0(0).write_raw(deadline as u32);
    RV_TIMER.compare_upper0(0).write_raw((deadline >> 32) as u32);
    RV_TIMER.intr_enable(0).reset().ie(true).reg.write();
}
