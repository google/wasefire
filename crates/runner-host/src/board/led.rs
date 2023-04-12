// Copyright 2022 Google LLC
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

const NUM_LEDS: usize = 1;

use wasefire_board_api as board;

impl board::led::Api for &mut crate::board::Board {
    fn count(&mut self) -> usize {
        NUM_LEDS
    }

    fn get(&mut self, led: usize) -> Result<bool, board::Error> {
        if NUM_LEDS <= led {
            return Err(board::Error::User);
        }
        Ok(self.state.lock().unwrap().led)
    }

    fn set(&mut self, led: usize, on: bool) -> Result<(), board::Error> {
        if NUM_LEDS <= led {
            return Err(board::Error::User);
        }
        println!("Led {led} is {}", if on { "on" } else { "off" });
        self.state.lock().unwrap().led = on;
        Ok(())
    }
}
