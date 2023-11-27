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

#![no_std]
wasefire::applet!();

use core::time::Duration;

use wasefire::sync::AtomicBool;
use wasefire::sync::Ordering::Relaxed;

fn main() -> ! {
    debug!("Waiting 5 seconds...");
    blink(Duration::from_secs(5));
    debug!("Waiting for a button press to reboot...");
    static PRESSED: AtomicBool = AtomicBool::new(false);
    let button = button::Listener::new(0, |event| match event {
        button::Pressed => PRESSED.store(true, Relaxed),
        button::Released => (),
    });
    scheduling::wait_until(|| PRESSED.load(Relaxed));
    drop(button);
    debug!("Waiting 3 seconds...");
    blink(Duration::from_secs(3));
    debug!("Rebooting...");
    platform::reboot();
}

fn blink(duration: Duration) {
    led::set(0, led::On);
    let blink = clock::Timer::new(|| led::set(0, !led::get(0)));
    blink.start(clock::Periodic, Duration::from_millis(300));
    clock::sleep(duration);
    drop(blink);
    led::set(0, led::Off);
}
